from __future__ import annotations

from pathlib import Path
from typing import Any
import hashlib
import json
import subprocess
import sys
import time

import modal


APP_NAME = "abyss-nes"
REPO_ROOT = Path("/root/ABYSS")
STATE_ROOT = Path("/mnt/abyss")
HF_HOME = STATE_ROOT / "hf-cache"


image = (
    modal.Image.from_registry("pytorch/pytorch:2.5.1-cuda12.4-cudnn9-runtime", add_python="3.11")
    .env(
        {
            "HF_HOME": str(HF_HOME),
            "HF_HUB_CACHE": str(HF_HOME / "hub"),
            "TRANSFORMERS_CACHE": str(HF_HOME / "transformers"),
            "PYTHONUNBUFFERED": "1",
        }
    )
    .pip_install("transformers", "datasets", "accelerate", "pyyaml", "huggingface_hub", "safetensors")
    .add_local_dir("src", str(REPO_ROOT / "src"), copy=True)
    .add_local_dir("config", str(REPO_ROOT / "config"), copy=True)
    .workdir(str(REPO_ROOT))
)

app = modal.App(APP_NAME)
state_volume = modal.Volume.from_name("abyss-nes-state", create_if_missing=True)


def ensure_src_path() -> None:
    src = str(REPO_ROOT / "src")
    if src not in sys.path:
        sys.path.insert(0, src)


def run(cmd: list[str]) -> str:
    completed = subprocess.run(cmd, text=True, capture_output=True, cwd=REPO_ROOT)
    if completed.returncode != 0:
        raise RuntimeError(
            "Command failed\n"
            f"cmd={cmd}\n"
            f"returncode={completed.returncode}\n"
            f"stdout={completed.stdout[-4000:]}\n"
            f"stderr={completed.stderr[-8000:]}"
        )
    return completed.stdout


@app.function(image=image, gpu="B200:1", volumes={STATE_ROOT: state_volume}, timeout=86400)
def evaluate_node(config_path: str, step: int, rank: int) -> dict[str, Any]:
    out = STATE_ROOT / "fitness" / f"step-{step:08d}" / f"rank-{rank:05d}.json"
    stdout = run(
        [
            "python",
            "-u",
            "src/train.py",
            "evaluate",
            "--config",
            config_path,
            "--step",
            str(step),
            "--rank",
            str(rank),
            "--out",
            str(out),
        ]
    )
    state_volume.commit()
    return {"rank": rank, "step": step, "out": str(out), "stdout_tail": stdout[-2000:]}


@app.function(image=image, volumes={STATE_ROOT: state_volume}, timeout=3600)
def evaluate_node_cpu(config_path: str, step: int, rank: int) -> dict[str, Any]:
    return evaluate_node.local(config_path, step, rank)


@app.function(image=image, volumes={STATE_ROOT: state_volume}, timeout=3600)
def coordinator_step(config_path: str, step: int) -> dict[str, Any]:
    ensure_src_path()
    from abyss_config import load_config, validate_config
    from nes import FitnessRecord, StepManifest, write_json

    cfg = load_config(REPO_ROOT / config_path)
    validate_config(cfg)
    population = int(cfg["nes"]["population_size"])
    quorum = int(cfg["nes"].get("quorum", population))

    evaluator = evaluate_node_cpu if cfg["model"].get("mock", False) else evaluate_node
    calls = [evaluator.spawn(config_path, step, rank) for rank in range(population)]
    fitness_dir = STATE_ROOT / "fitness" / f"step-{step:08d}"
    deadline = time.time() + int(cfg["modal"].get("node_timeout_seconds", 86400))

    while time.time() < deadline:
        state_volume.reload()
        records = read_fitness_records(fitness_dir)
        if len(records) >= quorum:
            break
        time.sleep(10)
    else:
        records = read_fitness_records(fitness_dir)

    if len(records) < quorum:
        raise RuntimeError(f"Step {step} quorum failed: {len(records)}/{quorum}")

    records = sorted(records, key=lambda item: item.rank)[:quorum]
    manifest = StepManifest(
        run_id=cfg["run_id"],
        step=step,
        sigma=float(cfg["nes"]["sigma"]),
        learning_rate=float(cfg["nes"]["learning_rate"]),
        participants=records,
    )
    manifest_path = STATE_ROOT / "manifests" / f"step-{step:08d}.json"
    write_json(manifest_path, manifest)
    state_volume.commit()

    completed = 0
    for call in calls:
        try:
            call.get(timeout=1)
            completed += 1
        except TimeoutError:
            pass

    return {
        "status": "manifest_written",
        "step": step,
        "records": len(records),
        "completed_calls": completed,
        "manifest": str(manifest_path),
    }


@app.function(image=image, gpu="B200:1", volumes={STATE_ROOT: state_volume}, timeout=86400)
def apply_update_node(config_path: str, step: int, rank: int) -> dict[str, Any]:
    state_volume.reload()
    manifest = STATE_ROOT / "manifests" / f"step-{step:08d}.json"
    save_dir = STATE_ROOT / "nodes" / f"rank-{rank:05d}" / f"step-{step + 1:08d}"
    stdout = run(
        [
            "python",
            "-u",
            "src/train.py",
            "apply-update",
            "--config",
            config_path,
            "--manifest",
            str(manifest),
            "--save-dir",
            str(save_dir),
        ]
    )
    state_volume.commit()
    return {"rank": rank, "step": step, "save_dir": str(save_dir), "stdout_tail": stdout[-2000:]}


@app.function(image=image, volumes={STATE_ROOT: state_volume}, timeout=3600)
def apply_update_node_cpu(config_path: str, step: int, rank: int) -> dict[str, Any]:
    return apply_update_node.local(config_path, step, rank)


@app.function(image=image, volumes={STATE_ROOT: state_volume}, timeout=3600)
def publish_checkpoint(step: int, source_rank: int = 0) -> dict[str, Any]:
    state_volume.reload()
    source = STATE_ROOT / "nodes" / f"rank-{source_rank:05d}" / f"step-{step:08d}"
    target = STATE_ROOT / "checkpoints" / f"step-{step:08d}"
    if not source.exists():
        raise FileNotFoundError(f"Missing source checkpoint: {source}")
    if target.exists():
        marker = target / "ABYSS_COMMITTED"
        if marker.exists():
            return {"status": "already_published", "checkpoint": str(target)}
    run(["python", "-c", f"import shutil; shutil.copytree({str(source)!r}, {str(target)!r}, dirs_exist_ok=True)"])
    checksum = checkpoint_fingerprint(target)
    meta = {
        "step": step,
        "checkpoint": str(target),
        "source_rank": source_rank,
        "fingerprint": checksum,
    }
    (target / "publish_meta.json").write_text(json.dumps(meta, indent=2, sort_keys=True), encoding="utf-8")
    marker = target / "ABYSS_COMMITTED"
    marker.write_text("committed\n", encoding="utf-8")
    latest_tmp = STATE_ROOT / "latest.json.tmp"
    latest = STATE_ROOT / "latest.json"
    latest_tmp.write_text(
        json.dumps(meta, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    latest_tmp.replace(latest)
    state_volume.commit()
    return {"status": "published", "checkpoint": str(target), "latest": str(latest)}


def checkpoint_fingerprint(path: Path) -> str:
    digest = hashlib.sha256()
    for item in sorted(path.rglob("*")):
        if item.is_file():
            rel = item.relative_to(path).as_posix()
            if rel in {"ABYSS_COMMITTED", "publish_meta.json"}:
                continue
            stat = item.stat()
            digest.update(rel.encode("utf-8"))
            digest.update(str(stat.st_size).encode("utf-8"))
    return digest.hexdigest()


@app.function(image=image, volumes={STATE_ROOT: state_volume}, timeout=86400)
def run_generation(config_path: str = "config/abyss72b_nes.yaml", start_step: int = 0, steps: int = 1):
    ensure_src_path()
    from abyss_config import load_config, validate_config

    cfg = load_config(REPO_ROOT / config_path)
    validate_config(cfg)
    population = int(cfg["nes"]["population_size"])
    results = []
    for step in range(start_step, start_step + steps):
        step_result = coordinator_step.remote(config_path, step)
        updater = apply_update_node_cpu if cfg["model"].get("mock", False) else apply_update_node
        apply_calls = [updater.spawn(config_path, step, rank) for rank in range(population)]
        applied = [call.get() for call in apply_calls]
        published = publish_checkpoint.remote(step + 1, 0)
        results.append({"coordinator": step_result, "applied": applied, "published": published})
    return results


@app.function(image=image, volumes={STATE_ROOT: state_volume}, timeout=3600)
def run_single_probe(config_path: str = "config/abyss_gpu_probe.yaml", step: int = 0, rank: int = 0):
    result = evaluate_node.remote(config_path, step, rank)
    return result


@app.function(image=image, gpu="B200:1", volumes={STATE_ROOT: state_volume}, timeout=86400)
def run_single_72b_candidate(config_path: str = "config/abyss72b_nes.yaml", step: int = 0, rank: int = 0):
    return evaluate_node.local(config_path, step, rank)


def read_fitness_records(path: Path):
    from nes import FitnessRecord

    records = []
    if not path.exists():
        return records
    for item in sorted(path.glob("rank-*.json")):
        raw = json.loads(item.read_text(encoding="utf-8"))
        records.append(FitnessRecord(**raw))
    return records


@app.local_entrypoint()
def main(
    config_path: str = "config/abyss72b_nes.yaml",
    start_step: int = 0,
    steps: int = 1,
    probe_only: bool = False,
    single_72b: bool = False,
    background: bool = False,
):
    if background:
        if single_72b:
            call = run_single_72b_candidate.spawn(config_path, start_step, 0)
        elif probe_only:
            call = run_single_probe.spawn(config_path, start_step, 0)
        else:
            call = run_generation.spawn(config_path, start_step, steps)
        print(json.dumps({"status": "spawned", "object_id": call.object_id}, indent=2, sort_keys=True))
    elif single_72b:
        result = run_single_72b_candidate.remote(config_path, start_step, 0)
        print(json.dumps(result, indent=2, sort_keys=True))
    elif probe_only:
        result = run_single_probe.remote(config_path, start_step, 0)
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        result = run_generation.remote(config_path, start_step, steps)
        print(json.dumps(result, indent=2, sort_keys=True))
