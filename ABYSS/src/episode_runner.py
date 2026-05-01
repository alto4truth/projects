from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from episode import EpisodeState, RewardConfig, compute_episode_reward
from tasks import PromptTask


@dataclass(frozen=True)
class EpisodeResult:
    task_id: str
    prompt: str
    answer: str
    hidden_token_count: int
    reward_model_score: float
    value_prediction: float
    value_mse: float
    final_reward: float


def run_episode(model, tokenizer, task: PromptTask, cfg: dict[str, Any], reward_model=None) -> EpisodeResult:
    print(f"[abyss] episode start task={task.id}", flush=True)
    answer, value_prediction = generate_answer(model, tokenizer, task.prompt, cfg)
    hidden_token_count = estimate_virtual_tokens(answer, cfg)
    visible_answer = strip_virtual_tokens(answer, cfg)
    rm_score = 1.0 if reward_model is None else reward_model.score(task.prompt, visible_answer)
    print(f"[abyss] episode terminal rm_score={rm_score} hidden_tokens={hidden_token_count}", flush=True)

    state = EpisodeState(prompt=task.prompt, hidden=["x " * hidden_token_count], visible=[visible_answer])
    reward_cfg = RewardConfig(vt_cost=float(cfg["fitness"].get("hidden_token_cost", 0.00001)))
    reward = compute_episode_reward(state, task_score=rm_score, config=reward_cfg)
    value_mse = (value_prediction - reward.reward) ** 2
    return EpisodeResult(
        task_id=task.id,
        prompt=task.prompt,
        answer=visible_answer,
        hidden_token_count=hidden_token_count,
        reward_model_score=rm_score,
        value_prediction=value_prediction,
        value_mse=value_mse,
        final_reward=reward.reward,
    )


def generate_answer(model, tokenizer, prompt: str, cfg: dict[str, Any]) -> tuple[str, float]:
    import torch

    max_new_tokens = int(cfg.get("episode", {}).get("max_new_tokens", 128))
    device = first_input_device(model)
    encoded = tokenizer(prompt, return_tensors="pt").to(device)
    with torch.no_grad():
        prompt_outputs = model(**encoded)
        value_prediction = float(prompt_outputs.values[:, -1].float().mean().item())
        output = model.generate(
            **encoded,
            max_new_tokens=max_new_tokens,
            do_sample=True,
            temperature=float(cfg.get("episode", {}).get("temperature", 0.7)),
            pad_token_id=tokenizer.eos_token_id,
        )
    generated = output[0, encoded["input_ids"].shape[1] :]
    return tokenizer.decode(generated, skip_special_tokens=False), value_prediction


def first_input_device(model):
    if hasattr(model, "base_model") and hasattr(model.base_model, "hf_device_map"):
        device_map = model.base_model.hf_device_map
        for name in ("model.embed_tokens", "transformer.wte", "base_model.model.embed_tokens"):
            if name in device_map:
                return device_map[name]
        for value in device_map.values():
            if isinstance(value, int):
                return f"cuda:{value}"
            if isinstance(value, str) and value not in {"cpu", "disk"}:
                return value
    return next(model.parameters()).device


def estimate_virtual_tokens(text: str, cfg: dict[str, Any]) -> int:
    vt = cfg["virtual_tokens"]
    prefix = str(vt["prefix"])
    suffix = str(vt["suffix"])
    count = 0
    start = 0
    while True:
        i = text.find(prefix, start)
        if i < 0:
            return count
        j = text.find(suffix, i + len(prefix))
        if j < 0:
            return count
        count += 1
        start = j + len(suffix)


def strip_virtual_tokens(text: str, cfg: dict[str, Any]) -> str:
    vt = cfg["virtual_tokens"]
    prefix = str(vt["prefix"])
    suffix = str(vt["suffix"])
    result = []
    start = 0
    while True:
        i = text.find(prefix, start)
        if i < 0:
            result.append(text[start:])
            return "".join(result).strip()
        result.append(text[start:i])
        j = text.find(suffix, i + len(prefix))
        if j < 0:
            return "".join(result).strip()
        start = j + len(suffix)
