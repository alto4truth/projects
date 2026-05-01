from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum


class ActionType(str, Enum):
    HIDDEN_CHUNK = "hidden_chunk"
    VISIBLE_CHUNK = "visible_chunk"
    TOOL_CALL = "tool_call"
    REVISE = "revise"
    SUMMARIZE = "summarize"
    TERMINATE = "terminate"


@dataclass(frozen=True)
class Action:
    type: ActionType
    text: str = ""
    hidden_tokens: int = 0
    visible_tokens: int = 0
    tool_calls: int = 0


@dataclass
class EpisodeState:
    prompt: str
    hidden: list[str] = field(default_factory=list)
    visible: list[str] = field(default_factory=list)
    tool_results: list[str] = field(default_factory=list)
    terminated: bool = False

    @property
    def answer(self) -> str:
        return "".join(self.visible).strip()

    @property
    def hidden_token_count(self) -> int:
        return sum(len(chunk.split()) for chunk in self.hidden)

    @property
    def visible_token_count(self) -> int:
        return sum(len(chunk.split()) for chunk in self.visible)

    @property
    def tool_call_count(self) -> int:
        return len(self.tool_results)

    def apply(self, action: Action) -> "EpisodeState":
        next_state = EpisodeState(
            prompt=self.prompt,
            hidden=list(self.hidden),
            visible=list(self.visible),
            tool_results=list(self.tool_results),
            terminated=self.terminated,
        )
        if action.type == ActionType.HIDDEN_CHUNK:
            next_state.hidden.append(action.text)
        elif action.type == ActionType.VISIBLE_CHUNK:
            next_state.visible.append(action.text)
        elif action.type == ActionType.TOOL_CALL:
            next_state.tool_results.append(action.text)
        elif action.type == ActionType.REVISE:
            next_state.visible = [action.text]
        elif action.type == ActionType.SUMMARIZE:
            next_state.hidden = [action.text]
        elif action.type == ActionType.TERMINATE:
            next_state.terminated = True
        else:
            raise ValueError(f"Unsupported action: {action.type}")
        return next_state


@dataclass(frozen=True)
class RewardConfig:
    vt_cost: float
    visible_cost: float = 0.0
    tool_cost: float = 0.0
    time_cost: float = 0.0
    failed_verifier_penalty: float = 0.0


@dataclass(frozen=True)
class RewardBreakdown:
    reward: float
    task_score: float
    virtual_token_penalty: float
    visible_token_penalty: float
    tool_penalty: float
    time_penalty: float
    verifier_penalty: float


def compute_episode_reward(
    state: EpisodeState,
    task_score: float,
    config: RewardConfig,
    search_steps: int = 0,
    verifier_failed: bool = False,
) -> RewardBreakdown:
    vt_penalty = state.hidden_token_count * config.vt_cost
    visible_penalty = state.visible_token_count * config.visible_cost
    tool_penalty = state.tool_call_count * config.tool_cost
    time_penalty = search_steps * config.time_cost
    verifier_penalty = config.failed_verifier_penalty if verifier_failed else 0.0
    reward = task_score - vt_penalty - visible_penalty - tool_penalty - time_penalty - verifier_penalty
    return RewardBreakdown(
        reward=reward,
        task_score=task_score,
        virtual_token_penalty=vt_penalty,
        visible_token_penalty=visible_penalty,
        tool_penalty=tool_penalty,
        time_penalty=time_penalty,
        verifier_penalty=verifier_penalty,
    )

