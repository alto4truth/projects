from __future__ import annotations

from pathlib import Path
from typing import Any
import json


def load_config(path: str | Path) -> dict[str, Any]:
    path = Path(path)
    try:
        import yaml  # type: ignore

        with path.open("r", encoding="utf-8") as f:
            return yaml.safe_load(f)
    except ModuleNotFoundError:
        return load_yaml_subset(path)


def validate_config(cfg: dict[str, Any]) -> None:
    required = [
        "run_id",
        "model_name",
        "checkpoint_dir",
        "state_dir",
        "model",
        "sequence",
        "virtual_tokens",
        "nes",
        "fitness",
    ]
    missing = [key for key in required if key not in cfg]
    if missing:
        raise ValueError(f"Missing config keys: {missing}")
    if int(cfg["nes"]["population_size"]) <= 0:
        raise ValueError("nes.population_size must be positive")
    if bool(cfg["nes"].get("antithetic", True)) and int(cfg["nes"]["population_size"]) % 2:
        raise ValueError("antithetic NES requires an even population_size")


def load_yaml_subset(path: Path) -> dict[str, Any]:
    root: dict[str, Any] = {}
    stack: list[tuple[int, dict[str, Any]]] = [(-1, root)]
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        if not raw_line.strip() or raw_line.lstrip().startswith("#"):
            continue
        indent = len(raw_line) - len(raw_line.lstrip(" "))
        line = raw_line.strip()
        key, sep, value = line.partition(":")
        if not sep:
            raise ValueError(f"Unsupported YAML line: {raw_line}")
        while indent <= stack[-1][0]:
            stack.pop()
        parent = stack[-1][1]
        if value.strip() == "":
            child: dict[str, Any] = {}
            parent[key] = child
            stack.append((indent, child))
        else:
            parent[key] = parse_scalar(value.strip())
    return root


def parse_scalar(value: str) -> Any:
    if value in {"true", "false"}:
        return value == "true"
    try:
        return json.loads(value)
    except json.JSONDecodeError:
        return value

