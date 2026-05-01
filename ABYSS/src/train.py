from __future__ import annotations

import argparse
import json
from pathlib import Path

from abyss_config import load_config, validate_config
from fitness import evaluate_candidate
from modeling import load_abyss_model, save_abyss_model
from nes import (
    FitnessRecord,
    StepManifest,
    apply_candidate_perturbation,
    apply_manifest_update,
    candidate_for_rank,
    read_manifest,
    write_json,
)
from reward_model import load_reward_model


def main() -> None:
    parser = argparse.ArgumentParser(description="ABYSS replicated distributed NES node.")
    sub = parser.add_subparsers(dest="command", required=True)

    dry = sub.add_parser("dry-run")
    dry.add_argument("--config", required=True)

    eval_cmd = sub.add_parser("evaluate")
    eval_cmd.add_argument("--config", required=True)
    eval_cmd.add_argument("--step", type=int, required=True)
    eval_cmd.add_argument("--rank", type=int, required=True)
    eval_cmd.add_argument("--out", required=True)

    update_cmd = sub.add_parser("apply-update")
    update_cmd.add_argument("--config", required=True)
    update_cmd.add_argument("--manifest", required=True)
    update_cmd.add_argument("--save-dir", required=True)

    args = parser.parse_args()
    cfg = load_and_validate(args.config)

    if args.command == "dry-run":
        print(json.dumps({"status": "dry_run_ok", "run_id": cfg["run_id"]}, indent=2))
    elif args.command == "evaluate":
        evaluate(cfg, args.step, args.rank, args.out)
    elif args.command == "apply-update":
        apply_update(cfg, args.manifest, args.save_dir)
    else:
        raise ValueError(args.command)


def load_and_validate(path: str) -> dict:
    cfg = load_config(path)
    validate_config(cfg)
    return cfg


def evaluate(cfg: dict, step: int, rank: int, out: str) -> None:
    print(f"[abyss] evaluate start step={step} rank={rank}", flush=True)
    if cfg["model"].get("mock", False):
        candidate = candidate_for_rank(cfg["run_id"], step, rank, int(cfg["nes"]["population_size"]))
        fitness = mock_fitness(candidate.seed, candidate.sign, rank, step)
        record = FitnessRecord(
            run_id=cfg["run_id"],
            step=step,
            rank=rank,
            seed=candidate.seed,
            sign=candidate.sign,
            fitness=fitness,
            diagnostics={"mock": True},
        )
        write_json(out, record)
        return

    model_source = resolve_model_source(cfg, step)
    print(f"[abyss] model_source={model_source}", flush=True)
    model, tokenizer, _ = load_abyss_model(cfg, model_source=model_source)
    model.eval()
    rm = load_reward_model(cfg)
    candidate = candidate_for_rank(cfg["run_id"], step, rank, int(cfg["nes"]["population_size"]))
    print(f"[abyss] applying perturbation seed={candidate.seed} sign={candidate.sign}", flush=True)
    apply_candidate_perturbation(model, candidate, float(cfg["nes"]["sigma"]), cfg["nes"])
    print("[abyss] evaluating candidate fitness", flush=True)
    fitness, diagnostics = evaluate_candidate(model, tokenizer, cfg, rank, step, reward_model=rm)
    record = FitnessRecord(
        run_id=cfg["run_id"],
        step=step,
        rank=rank,
        seed=candidate.seed,
        sign=candidate.sign,
        fitness=fitness,
        diagnostics=diagnostics,
    )
    write_json(out, record)
    print(f"[abyss] evaluate done fitness={fitness} out={out}", flush=True)


def apply_update(cfg: dict, manifest_path: str, save_dir: str) -> None:
    if cfg["model"].get("mock", False):
        manifest = read_manifest(manifest_path)
        save_dir_path = Path(save_dir)
        save_dir_path.mkdir(parents=True, exist_ok=True)
        write_json(save_dir_path / "manifest.json", manifest)
        write_json(save_dir_path / "checkpoint_meta.json", {
            "run_id": cfg["run_id"],
            "source": "mock",
            "step": manifest.step + 1,
            "base_step": manifest.step,
        })
        (save_dir_path / "mock_checkpoint.txt").write_text("mock checkpoint\n", encoding="utf-8")
        return

    model_source = resolve_model_source(cfg, manifest.step)
    model, tokenizer, _ = load_abyss_model(cfg, model_source=model_source)
    manifest = read_manifest(manifest_path)
    if manifest.run_id != cfg["run_id"]:
        raise ValueError(f"Manifest run_id {manifest.run_id} != config run_id {cfg['run_id']}")
    apply_manifest_update(model, manifest, cfg["nes"])
    save_dir_path = Path(save_dir)
    save_dir_path.mkdir(parents=True, exist_ok=True)
    save_abyss_model(model, tokenizer, save_dir_path)
    write_json(save_dir_path / "manifest.json", manifest)
    write_json(save_dir_path / "checkpoint_meta.json", {
        "run_id": cfg["run_id"],
        "source": model_source,
        "step": manifest.step + 1,
        "base_step": manifest.step,
    })


def build_manifest(cfg: dict, step: int, records: list[FitnessRecord]) -> StepManifest:
    return StepManifest(
        run_id=cfg["run_id"],
        step=step,
        sigma=float(cfg["nes"]["sigma"]),
        learning_rate=float(cfg["nes"]["learning_rate"]),
        participants=records,
    )


def mock_fitness(seed: int, sign: int, rank: int, step: int) -> float:
    # Deterministic toy objective for protocol smoke tests.
    return ((seed % 10_000) / 10_000.0) * sign - 0.001 * rank + 0.0001 * step


def resolve_model_source(cfg: dict, step: int) -> str:
    checkpoint = Path(cfg["state_dir"]) / "checkpoints" / f"step-{step:08d}"
    if checkpoint.exists():
        return str(checkpoint)
    genesis = Path(cfg["checkpoint_dir"])
    if genesis.exists():
        return str(genesis)
    return str(cfg["model_name"])


if __name__ == "__main__":
    main()
