from __future__ import annotations

from dataclasses import dataclass, asdict
from hashlib import blake2b
from pathlib import Path
from typing import Any
import json
import math


@dataclass(frozen=True)
class Candidate:
    step: int
    rank: int
    seed: int
    sign: int


@dataclass(frozen=True)
class FitnessRecord:
    run_id: str
    step: int
    rank: int
    seed: int
    sign: int
    fitness: float
    diagnostics: dict[str, Any]


@dataclass(frozen=True)
class StepManifest:
    run_id: str
    step: int
    sigma: float
    learning_rate: float
    participants: list[FitnessRecord]


def candidate_for_rank(run_id: str, step: int, rank: int, population_size: int) -> Candidate:
    if rank < 0 or rank >= population_size:
        raise ValueError(f"rank {rank} outside population {population_size}")
    pair = rank // 2
    sign = 1 if rank % 2 == 0 else -1
    seed = stable_seed(run_id, step, pair)
    return Candidate(step=step, rank=rank, seed=seed, sign=sign)


def stable_seed(run_id: str, step: int, pair: int) -> int:
    h = blake2b(f"{run_id}:{step}:{pair}".encode("utf-8"), digest_size=8).digest()
    return int.from_bytes(h, "little") & 0x7FFFFFFFFFFFFFFF


def rank_utilities(records: list[FitnessRecord]) -> dict[int, float]:
    ordered = sorted(records, key=lambda record: record.fitness)
    n = len(ordered)
    utilities: dict[int, float] = {}
    for idx, record in enumerate(ordered):
        centered_rank = idx - (n - 1) / 2
        utilities[record.rank] = centered_rank / max((n - 1) / 2, 1)
    return utilities


def apply_manifest_update(model, manifest: StepManifest, perturbation_cfg: dict[str, Any]) -> None:
    import torch

    utilities = rank_utilities(manifest.participants)
    with torch.no_grad():
        for param_index, param in enumerate(model.parameters()):
            if not param.requires_grad or not param.is_floating_point():
                continue
            update = torch.zeros_like(param)
            for record in manifest.participants:
                noise = make_noise_like(param, record.seed, param_index, perturbation_cfg)
                update.add_(noise, alpha=utilities[record.rank] * record.sign)
            scale = manifest.learning_rate / (len(manifest.participants) * manifest.sigma)
            param.add_(update, alpha=scale)


def apply_candidate_perturbation(model, candidate: Candidate, sigma: float, perturbation_cfg: dict[str, Any]) -> None:
    import torch

    with torch.no_grad():
        for param_index, param in enumerate(model.parameters()):
            if not param.requires_grad or not param.is_floating_point():
                continue
            noise = make_noise_like(param, candidate.seed, param_index, perturbation_cfg)
            param.add_(noise, alpha=sigma * candidate.sign)


def make_noise_like(param, seed: int, param_index: int, perturbation_cfg: dict[str, Any]):
    import torch

    generator = torch.Generator(device=param.device)
    generator.manual_seed((seed + 1_000_003 * param_index) % (2**63 - 1))
    if perturbation_cfg.get("perturbation") == "blockwise" and param.ndim >= 2:
        block = int(perturbation_cfg.get("block_size_tensors", 8))
        rows = math.ceil(param.shape[0] / block)
        base = torch.randn((rows,) + param.shape[1:], generator=generator, device=param.device, dtype=param.dtype)
        return base.repeat_interleave(block, dim=0)[: param.shape[0]]
    return torch.randn(param.shape, generator=generator, device=param.device, dtype=param.dtype)


def write_json(path: str | Path, value: Any) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as f:
        json.dump(to_jsonable(value), f, indent=2, sort_keys=True)


def read_manifest(path: str | Path) -> StepManifest:
    raw = json.loads(Path(path).read_text(encoding="utf-8"))
    participants = [FitnessRecord(**item) for item in raw["participants"]]
    return StepManifest(
        run_id=raw["run_id"],
        step=int(raw["step"]),
        sigma=float(raw["sigma"]),
        learning_rate=float(raw["learning_rate"]),
        participants=participants,
    )


def to_jsonable(value: Any) -> Any:
    if hasattr(value, "__dataclass_fields__"):
        return asdict(value)
    if isinstance(value, list):
        return [to_jsonable(item) for item in value]
    if isinstance(value, dict):
        return {key: to_jsonable(item) for key, item in value.items()}
    return value

