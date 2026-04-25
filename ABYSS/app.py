from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any

import modal


APP_NAME = "abyss"
REPO_ROOT = Path("/root/ABYSS")
DATA_DIR = Path("/mnt/data")
MODELS_DIR = Path("/mnt/models")


image = (
    modal.Image.from_registry("pytorch/pytorch:2.1.0-cuda12.1-cudnn9-runtime", add_python="3.11")
    .add_local_dir("src", str(REPO_ROOT / "src"), copy=True)
    .add_local_dir("config", str(REPO_ROOT / "config"), copy=True)
    .run_commands(
        "pip install transformers datasets accelerate huggingface_hub",
    )
    .workdir(str(REPO_ROOT))
)

app = modal.App(APP_NAME)
data_volume = modal.Volume.from_name("abyss-data", create_if_missing=True, version=2)
models_volume = modal.Volume.from_name("abyss-models", create_if_missing=True, version=2)


def run_command(cmd: list[str]) -> str:
    completed = subprocess.run(cmd, cwd=REPO_ROOT, check=True, capture_output=True, text=True)
    return completed.stdout


@app.function(image=image, volumes={DATA_DIR: data_volume, MODELS_DIR: models_volume}, gpu="H100", timeout=86400)
def train(config: dict[str, Any]):
    print(f"Training with config: {config}")
    return {"status": "started", "config": config}


@app.function(image=image, volumes={MODELS_DIR: models_volume}, gpu="H100", timeout=3600)
def generate(prompt: str, max_tokens: int = 256):
    print(f"Generating: {prompt}")
    return {"prompt": prompt, "generated": "..."}


@app.local_entrypoint()
def main():
    train({"epochs": 100, "batch_size": 32})