from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable
import json


@dataclass(frozen=True)
class PromptTask:
    id: str
    prompt: str


def load_prompt_tasks(cfg: dict[str, Any], limit: int | None = None) -> list[PromptTask]:
    task_cfg = cfg.get("tasks", {})
    source = task_cfg.get("source", "builtin")
    if source == "builtin":
        tasks = builtin_tasks()
    elif source == "jsonl":
        tasks = load_jsonl_tasks(Path(task_cfg["path"]), task_cfg.get("prompt_field", "prompt"))
    elif source == "hf":
        tasks = load_hf_tasks(
            task_cfg["name"],
            task_cfg.get("split", "train"),
            task_cfg.get("prompt_field", "instruction"),
            task_cfg.get("streaming", True),
        )
    else:
        raise ValueError(f"Unsupported task source: {source}")
    if limit is not None:
        return list(tasks[:limit])
    return list(tasks)


def builtin_tasks() -> list[PromptTask]:
    return [
        PromptTask("builtin_0001", "Explain why the sky is blue in one concise paragraph."),
        PromptTask("builtin_0002", "Write a short Python function that adds two numbers."),
        PromptTask("builtin_0003", "Give three practical tips for debugging a failing test."),
    ]


def load_jsonl_tasks(path: Path, prompt_field: str) -> list[PromptTask]:
    tasks = []
    with path.open("r", encoding="utf-8") as f:
        for idx, line in enumerate(f):
            if not line.strip():
                continue
            raw = json.loads(line)
            tasks.append(PromptTask(str(raw.get("id", f"jsonl_{idx:06d}")), str(raw[prompt_field])))
    return tasks


def load_hf_tasks(name: str, split: str, prompt_field: str, streaming: bool) -> list[PromptTask]:
    from datasets import load_dataset

    ds = load_dataset(name, split=split, streaming=streaming)
    tasks = []
    for idx, item in enumerate(ds):
        prompt = extract_prompt(item, prompt_field)
        if prompt:
            tasks.append(PromptTask(f"hf_{idx:08d}", prompt))
        if len(tasks) >= 10_000:
            break
    return tasks


def extract_prompt(item: dict[str, Any], prompt_field: str) -> str:
    value = item.get(prompt_field)
    if isinstance(value, str):
        return value
    if isinstance(value, list) and value:
        first = value[0]
        if isinstance(first, dict):
            return str(first.get("content", ""))
    return ""

