from __future__ import annotations

from typing import Any

from episode_runner import run_episode
from tasks import load_prompt_tasks


def evaluate_candidate(
    model,
    tokenizer,
    cfg: dict[str, Any],
    rank: int,
    step: int,
    reward_model=None,
) -> tuple[float, dict[str, Any]]:
    tasks = load_prompt_tasks(cfg, limit=int(cfg["fitness"].get("tasks_per_step", 1)))
    if not tasks:
        raise ValueError("No prompt tasks available")
    rewards = []
    value_losses = []
    diagnostics = []
    for index, task in enumerate(tasks):
        if index % max(int(cfg["nes"]["population_size"]), 1) != rank % max(int(cfg["nes"]["population_size"]), 1):
            continue
        result = run_episode(model, tokenizer, task, cfg, reward_model=reward_model)
        rewards.append(result.final_reward)
        value_losses.append(result.value_mse)
        diagnostics.append({
            "task_id": result.task_id,
            "reward_model_score": result.reward_model_score,
            "hidden_token_count": result.hidden_token_count,
            "value_prediction": result.value_prediction,
            "value_mse": result.value_mse,
            "final_reward": result.final_reward,
        })
    if not rewards:
        task = tasks[rank % len(tasks)]
        result = run_episode(model, tokenizer, task, cfg, reward_model=reward_model)
        rewards.append(result.final_reward)
        value_losses.append(result.value_mse)
        diagnostics.append({
            "task_id": result.task_id,
            "reward_model_score": result.reward_model_score,
            "hidden_token_count": result.hidden_token_count,
            "value_prediction": result.value_prediction,
            "value_mse": result.value_mse,
            "final_reward": result.final_reward,
        })
    reward_mean = sum(rewards) / len(rewards)
    value_mse = sum(value_losses) / len(value_losses)
    value_weight = float(cfg["fitness"].get("value_loss_weight", 0.1))
    fitness = reward_mean - value_weight * value_mse
    return fitness, {
        "episodes": diagnostics,
        "reward_mean": reward_mean,
        "value_mse": value_mse,
        "value_loss_weight": value_weight,
        "fitness": fitness,
    }
