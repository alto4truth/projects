#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import json
import shutil
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from abyss_config import load_config, validate_config
from nes import FitnessRecord, StepManifest, write_json
from train import apply_update, evaluate


def main() -> None:
    cfg_path = ROOT / "config" / "abyss_smoke_nes.yaml"
    cfg = load_config(cfg_path)
    validate_config(cfg)

    state = Path("/tmp/abyss-local-smoke")
    if state.exists():
        shutil.rmtree(state)
    (state / "fitness").mkdir(parents=True)
    (state / "manifests").mkdir(parents=True)
    (state / "checkpoints" / "step-00000000").mkdir(parents=True)
    (state / "checkpoints" / "step-00000000" / "mock_checkpoint.txt").write_text(
        "genesis\n",
        encoding="utf-8",
    )

    cfg = dict(cfg)
    cfg["state_dir"] = str(state)
    cfg["checkpoint_dir"] = str(state / "checkpoints" / "step-00000000")

    population = int(cfg["nes"]["population_size"])
    steps = 3
    for step in range(steps):
        records = []
        step_dir = state / "fitness" / f"step-{step:08d}"
        step_dir.mkdir(parents=True, exist_ok=True)
        for rank in range(population):
            out = step_dir / f"rank-{rank:05d}.json"
            evaluate(cfg, step, rank, str(out))
            records.append(FitnessRecord(**json.loads(out.read_text(encoding="utf-8"))))

        manifest = StepManifest(
            run_id=cfg["run_id"],
            step=step,
            sigma=float(cfg["nes"]["sigma"]),
            learning_rate=float(cfg["nes"]["learning_rate"]),
            participants=records,
        )
        manifest_path = state / "manifests" / f"step-{step:08d}.json"
        write_json(manifest_path, manifest)

        checkpoint = state / "checkpoints" / f"step-{step + 1:08d}"
        apply_update(cfg, str(manifest_path), str(checkpoint))
        assert (checkpoint / "checkpoint_meta.json").exists()
        assert (checkpoint / "manifest.json").exists()

    latest = state / "checkpoints" / f"step-{steps:08d}"
    assert latest.exists()
    print(f"local_nes_smoke_ok {latest}")


if __name__ == "__main__":
    main()

