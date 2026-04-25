"""ABYSS Training with NES - Main entry point."""

import os
import torch
import torch.nn as nn
import torch.distributed as dist
from torch.utils.data import DataLoader, IterableDataset
from datasets import load_dataset
import modal
from typing import Optional
import json


APP_NAME = "abyss-trainer"


class ABYSSDataset(IterableDataset):
    """Streaming dataset from HuggingFace."""
    
    def __init__(self, dataset_name: str, split: str = "train"):
        self.dataset_name = dataset_name
        self.split = split
    
    def __iter__(self):
        ds = load_dataset(self.dataset_name, split=self.split, streaming=True)
        for item in ds:
            yield item["text"]


class ABYSS:
    """ABYSS Model - 70B Transformer."""
    
    def __init__(
        self,
        vocab_size: int = 51200,
        d_model: int = 8192,
        n_heads: int = 64,
        n_layers: 80,
        d_ff: int = 32768,
        max_seq_len: int = 8192,
    ):
        self.vocab_size = vocab_size
        self.d_model = d_model
        
        self.token_emb = nn.Embedding(vocab_size, d_model)
        self.pos_emb = nn.Embedding(max_seq_len, d_model)
        
        self.layers = nn.ModuleList([
            TransformerLayer(d_model, n_heads, d_ff)
            for _ in range(n_layers)
        ])
        
        self.norm = nn.LayerNorm(d_model)
        self.lm_head = nn.Linear(d_model, vocab_size, bias=False)
        self.lm_head.weight = self.token_emb.weight
    
    def forward(self, input_ids, attention_mask=None):
        b, seq_len = input_ids.shape
        x = self.token_emb(input_ids) + self.pos_emb(torch.arange(seq_len, device=input_ids.device))
        
        causal = torch.triu(torch.ones(seq_len, seq_len, device=x.device) * float("-inf"), diagonal=1)
        
        for layer in self.layers:
            x = layer(x, causal)
        
        return self.norm(x)
    
    def generate(self, input_ids, max_tokens: int = 100):
        self.eval()
        for _ in range(max_tokens):
            logits = self.forward(input_ids)[:, -1]
            next_token = logits.argmax(dim=-1, keepdim=True)
            input_ids = torch.cat([input_ids, next_token], dim=1)
        return input_ids


class TransformerLayer(nn.Module):
    def __init__(self, d_model: int, n_heads: int, d_ff: int):
        super().__init__()
        self.attn = nn.MultiheadAttention(d_model, n_heads, batch_first=True, device="cuda")
        self.ff = nn.Sequential(
            nn.Linear(d_model, d_ff, device="cuda"),
            nn.GELU(),
            nn.Linear(d_ff, d_model, device="cuda"),
        )
        self.norm1 = nn.LayerNorm(d_model, device="cuda")
        self.norm2 = nn.LayerNorm(d_model, device="cuda")
    
    def forward(self, x, mask):
        attn, _ = self.attn(x, x, x, attn_mask=mask)
        x = self.norm1(x + attn)
        x = self.norm2(x + self.ff(x))
        return x


def count_parameters(model: nn.Module) -> int:
    return sum(p.numel() for p in model.parameters())


def get_theta(model: nn.Module) -> torch.Tensor:
    """Flatten all parameters to single tensor."""
    return torch.cat([p.data.flatten() for p in model.parameters()])


def set_theta(model: nn.Module, theta: torch.Tensor):
    """Set parameters from flattened tensor."""
    offset = 0
    for p in model.parameters():
        numel = p.numel()
        p.data = theta[offset:offset+numel].reshape(p.shape).to(p.device)
        offset += numel


def compute_fitness(model: nn.Module, batch) -> float:
    """Compute fitness = negative loss."""
    model.eval()
    with torch.no_grad():
        input_ids = batch["input_ids"].to("cuda")
        logits = model(input_ids)
        loss = nn.functional.cross_entropy(
            logits.view(-1, logits.size(-1)),
            input_ids.view(-1),
            ignore=0,
        )
    return -loss.item()


class NES:
    """NES optimizer for ABYSS."""
    
    def __init__(
        self,
        model: nn.Module,
        popsize: int = 256,
        lr: float = 0.01,
        sigma: float = 0.01,
    ):
        self.model = model
        self.popsize = popsize
        self.lr = lr
        self.sigma = sigma
        self.theta = get_theta(model)
    
    def step(self, fitness_fn):
        """Single NES step."""
        # Generate noise on-the-fly
        fitnesses = []
        
        for _ in range(self.popsize):
            noise = torch.randn_like(self.theta) * self.sigma
            theta_candidate = self.theta + noise
            
            set_theta(self.model, theta_candidate)
            fitness = fitness_fn(self.model)
            fitnesses.append(fitness)
        
        # Ranking
        fitnesses = torch.tensor(fitnesses)
        ranks = torch.argsort(torch.argsort(fitnesses)).float()
        normalized_ranks = (ranks / self.popsize - 0.5) * 2
        
        # Estimate gradient
        gradient = torch.zeros_like(self.theta)
        for i in range(self.popsize):
            noise = torch.randn_like(self.theta) * self.sigma
            gradient += normalized_ranks[i] * noise
        
        gradient /= self.popsize
        self.theta += gradient * self.lr
        
        set_theta(self.model, self.theta)
        return fitnesses.mean().item()


def distributed_step(model, fitness_fn, world_size: int):
    """Distributed NES across GPUs."""
    if world_size == 1:
        return NES(model).step(fitness_fn)
    
    local_popsize = 256 // world_size
    local_fitnesses = []
    
    for _ in range(local_popsize):
        noise = torch.randn_like(get_theta(model)) * 0.01
        set_theta(model, get_theta(model) + noise)
        local_fitness = fitness_fn(model)
        local_fitnesses.append(local_fitness)
    
    local_fitnesses = torch.tensor(local_fitnesses, device="cuda")
    
    all_fitness = torch.zeros(world_size, device="cuda")
    dist.all_gather(all_fitness, local_fitnesses.mean())
    
    global_rank = torch.argsort(torch.argsort(all_fitness.mean()))[0]
    
    return global_rank.item()


@modal.App.function(gpu="H100:8", timeout=86400)
def train(config: dict):
    """Main training function on Modal."""
    world_size = int(os.environ.get("WORLD_SIZE", "1"))
    rank = int(os.environ.get("RANK", "0"))
    
    if world_size > 1:
        dist.init_process_group(backend="nccl")
    
    # Initialize model
    model = ABYSS(
        d_model=config.get("d_model", 8192),
        n_heads=config.get("n_heads", 64),
        n_layers=config.get("n_layers", 80),
    ).to("cuda")
    
    model = torch.nn.DataParallel(model) if world_size > 1 else model
    
    print(f"[Rank {rank}] Model params: {count_parameters(model):,}")
    
    # Dataset
    dataset = ABYSSDataset(config.get("dataset", "HuggingFaceFW/fineweb-edu"))
    
    # Use tokenizer
    from transformers import AutoTokenizer
    tokenizer = AutoTokenizer.from_pretrained("gpt2")
    
    # Training loop
    nes = NES(model, popsize=config.get("popsize", 256))
    
    for epoch in range(config.get("epochs", 10)):
        dataloader = DataLoader(dataset, batch_size=config.get("batch_size", 1))
        
        for batch in dataloader:
            batch = tokenizer(batch["text"], return_tensors="pt", truncation=True, max_length=4096)
            
            def fitness_fn(m):
                return compute_fitness(m, batch)
            
            avg_fitness = distributed_step(model, fitness_fn, world_size)
            
            if rank == 0:
                print(f"Epoch {epoch}, fitness: {avg_fitness:.4f}")
    
    if world_size > 1:
        dist.destroy_process_group()
    
    return {"status": "done", "final_fitness": avg_fitness}


@modal.local_entrypoint()
def main():
    config = {
        "dataset": "HuggingFaceFW/fineweb-edu",
        "d_model": 8192,
        "n_heads": 64,
        "n_layers": 80,
        "popsize": 256,
        "batch_size": 1,
        "epochs": 10,
    }
    train.spawn(config)
    print("Training started!")