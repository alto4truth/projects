import os
import torch
import torch.distributed as dist
from torch.nn.parallel import DistributedDataParallel as DDP
from torch.utils.data import DistributedSampler
import subprocess


def setup_distributed():
    """Initialize distributed training."""
    if "RANK" in os.environ:
        rank = int(os.environ["RANK"])
        local_rank = int(os.environ.get("LOCAL_RANK", 0))
        world_size = int(os.environ["WORLD_SIZE"])
    else:
        rank = local_rank = 0
        world_size = 1
    
    if world_size > 1:
        dist.init_process_group(backend="nccl")
        torch.cuda.set_device(local_rank)
    
    return rank, local_rank, world_size


def cleanup_distributed():
    if dist.is_initialized():
        dist.destroy_process_group()


class Trainer:
    def __init__(self, model, optimizer, world_size):
        self.model = DDP(model) if world_size > 1 else model
        self.optimizer = optimizer
        self.world_size = world_size
    
    def train_step(self, batch):
        self.model.train()
        loss = self.model(**batch).loss
        loss.backward()
        self.optimizer.step()
        self.optimizer.zero_grad()
        return loss.item()


def launch_training(script_path: str, n_gpus: int = 8):
    """Launch distributed training on Modal with multiple GPUs."""
    cmd = [
        "torchrun",
        f"--nproc_per_node={n_gpus}",
        script_path
    ]
    subprocess.run(cmd)


if __name__ == "__main__":
    rank, local_rank, world_size = setup_distributed()
    print(f"Rank: {rank}/{world_size}")
    cleanup_distributed()