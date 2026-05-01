#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "src"))

from episode import Action, ActionType, EpisodeState, RewardConfig, compute_episode_reward
from episode_runner import estimate_virtual_tokens, strip_virtual_tokens
from mcts import MCTSNode, should_stop


def main() -> None:
    state = EpisodeState("What is 40 + 2?")
    state = state.apply(Action(ActionType.HIDDEN_CHUNK, "think think think"))
    state = state.apply(Action(ActionType.VISIBLE_CHUNK, "42"))
    reward = compute_episode_reward(state, 1.0, RewardConfig(vt_cost=0.00001))
    assert reward.reward == 1.0 - 3 * 0.00001

    root = MCTSNode(EpisodeState("x"))
    top1 = MCTSNode(root.state.apply(Action(ActionType.VISIBLE_CHUNK, "a")))
    top2 = MCTSNode(root.state.apply(Action(ActionType.VISIBLE_CHUNK, "b")))
    top1.visits = 95
    top2.visits = 5
    root.children = [top1, top2]
    root.visits = 100
    decision = should_stop(root, min_visits=32, certainty_threshold=0.95)
    assert decision.should_stop
    assert decision.certainty == 0.95
    cfg = {"virtual_tokens": {"prefix": "<|abyss_latent_", "suffix": "|>"}}
    text = "hello <|abyss_latent_1|>hidden<|abyss_latent_2|> world"
    assert estimate_virtual_tokens(text, cfg) == 2
    assert strip_virtual_tokens(text, cfg) == "hello hidden world"
    print("mcts_checks_ok")


if __name__ == "__main__":
    main()
