import torch
import torch.nn as nn
from typing import Optional


class TransformerLayer(nn.Module):
    def __init__(
        self,
        d_model: int = 4096,
        n_heads: int = 32,
        d_ff: int = 16384,
        dropout: float = 0.1,
    ):
        super().__init__()
        self.attn = nn.MultiheadAttention(d_model, n_heads, dropout, batch_first=True)
        self.ff = nn.Sequential(
            nn.Linear(d_model, d_ff),
            nn.GELU(),
            nn.Linear(d_ff, d_model),
        )
        self.norm1 = nn.LayerNorm(d_model)
        self.norm2 = nn.LayerNorm(d_model)
        self.dropout = nn.Dropout(dropout)
    
    def forward(self, x, mask=None):
        # Attention with residual
        attn_out, _ = self.attn(x, x, x, attn_mask=mask)
        x = self.norm1(x + self.dropout(attn_out))
        
        # Feedforward with residual
        ff_out = self.ff(x)
        x = self.norm2(x + self.dropout(ff_out))
        return x


class ABYSSModel(nn.Module):
    """ABYSS Transformer for AGI."""
    
    def __init__(
        self,
        vocab_size: int = 51200,
        d_model: int = 4096,
        n_heads: int = 32,
        n_layers: int = 32,
        d_ff: int = 16384,
        dropout: float = 0.1,
        max_seq_len: int = 8192,
    ):
        super().__init__()
        self.vocab_size = vocab_size
        self.d_model = d_model
        
        self.token_emb = nn.Embedding(vocab_size, d_model)
        self.pos_emb = nn.Embedding(max_seq_len, d_model)
        self.dropout = nn.Dropout(dropout)
        
        self.layers = nn.ModuleList([
            TransformerLayer(d_model, n_heads, d_ff, dropout)
            for _ in range(n_layers)
        ])
        
        self.norm = nn.LayerNorm(d_model)
        self.lm_head = nn.Linear(d_model, vocab_size, bias=False)
        
        # Tie weights
        self.lm_head.weight = self.token_emb.weight
    
    def forward(self, input_ids, attention_mask=None):
        b, seq_len = input_ids.shape
        
        # Token + positional embeddings
        positions = torch.arange(seq_len, device=input_ids.device)
        x = self.token_emb(input_ids) + self.pos_emb(positions)
        x = self.dropout(x)
        
        # Causal mask
        causal_mask = torch.triu(
            torch.ones(seq_len, seq_len, device=x.device) * float("-inf"),
            diagonal=1
        )
        
        # Forward through layers
        for layer in self.layers:
            x = layer(x, causal_mask)
        
        x = self.norm(x)
        logits = self.lm_head(x)
        
        return logits
    
    def generate(self, input_ids, max_new_tokens, temperature=0.7):
        """Simple greedy generation."""
        self.eval()
        for _ in range(max_new_tokens):
            logits = self.forward(input_ids)
            next_token_logits = logits[:, -1, :] / temperature
            next_token = next_token_logits.argmax(dim=-1, keepdim=True)
            input_ids = torch.cat([input_ids, next_token], dim=1)
        return input_ids


def count_parameters(model: nn.Module) -> int:
    return sum(p.numel() for p in model.parameters())


# Config
CONFIG = {
    "vocab_size": 51200,
    "d_model": 4096,
    "n_heads": 32,
    "n_layers": 32,
    "d_ff": 16384,
    "max_seq_len": 8192,
}


if __name__ == "__main__":
    model = ABYSSModel(**CONFIG)
    print(f"Parameters: {count_parameters(model):,}")