#!/usr/bin/env python3
"""Download datasets from HuggingFace for ABYSS training."""

from datasets import load_dataset
import sys

def download_fineweb_edu():
    print("Downloading FineWeb-Edu...")
    ds = load_dataset("HuggingFaceFW/fineweb-edu", split="train")
    ds.save_to_disk("/mnt/data/fineweb-edu")
    print(f"Saved {len(ds)} samples")

def download_dolma_sample():
    print("Downloading Dolma v1.6-sample...")
    ds = load_dataset("allenai/dolma", "v1_6-sample", split="train")
    ds.save_to_disk("/mnt/data/dolma-v1.6-sample")
    print(f"Saved {len(ds)} samples")

def download_openorca():
    print("Downloading OpenOrca...")
    ds = load_dataset("Open-Orca/OpenOrca", split="train[:100000]")
    ds.save_to_disk("/mnt/data/openorca")
    print(f"Saved {len(ds)} samples")

if __name__ == "__main__":
    dataset = sys.argv[1] if len(sys.argv) > 1 else "fineweb-edu"
    
    if dataset == "fineweb-edu":
        download_fineweb_edu()
    elif dataset == "dolma":
        download_dolma_sample()
    elif dataset == "openorca":
        download_openorca()
    else:
        print(f"Unknown: {dataset}")