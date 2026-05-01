from __future__ import annotations

from dataclasses import dataclass, field
import math

from episode import Action, ActionType, EpisodeState


@dataclass
class MCTSNode:
    state: EpisodeState
    action: Action | None = None
    prior: float = 1.0
    visits: int = 0
    value_sum: float = 0.0
    children: list["MCTSNode"] = field(default_factory=list)

    @property
    def value(self) -> float:
        return self.value_sum / self.visits if self.visits else 0.0


@dataclass(frozen=True)
class StopDecision:
    should_stop: bool
    certainty: float
    top1_visits: int
    top2_visits: int
    root_visits: int
    reason: str


def visit_certainty(root: MCTSNode) -> StopDecision:
    visits = sorted((child.visits for child in root.children), reverse=True)
    top1 = visits[0] if visits else 0
    top2 = visits[1] if len(visits) > 1 else 0
    denom = top1 + top2
    certainty = top1 / denom if denom else 0.0
    return StopDecision(
        should_stop=False,
        certainty=certainty,
        top1_visits=top1,
        top2_visits=top2,
        root_visits=root.visits,
        reason="not_evaluated",
    )


def should_stop(root: MCTSNode, min_visits: int, certainty_threshold: float) -> StopDecision:
    decision = visit_certainty(root)
    if decision.root_visits < min_visits:
        return StopDecision(
            should_stop=False,
            certainty=decision.certainty,
            top1_visits=decision.top1_visits,
            top2_visits=decision.top2_visits,
            root_visits=decision.root_visits,
            reason="min_visits",
        )
    if decision.certainty >= certainty_threshold:
        return StopDecision(
            should_stop=True,
            certainty=decision.certainty,
            top1_visits=decision.top1_visits,
            top2_visits=decision.top2_visits,
            root_visits=decision.root_visits,
            reason="visit_certainty",
        )
    return StopDecision(
        should_stop=False,
        certainty=decision.certainty,
        top1_visits=decision.top1_visits,
        top2_visits=decision.top2_visits,
        root_visits=decision.root_visits,
        reason="uncertain",
    )


def puct_score(parent: MCTSNode, child: MCTSNode, exploration_c: float) -> float:
    exploration = exploration_c * child.prior * math.sqrt(max(parent.visits, 1)) / (1 + child.visits)
    return child.value + exploration


def default_macro_actions() -> list[Action]:
    return [
        Action(ActionType.HIDDEN_CHUNK, text="<latent reasoning>", hidden_tokens=2),
        Action(ActionType.VISIBLE_CHUNK, text="42", visible_tokens=1),
        Action(ActionType.TOOL_CALL, text="verifier", tool_calls=1),
        Action(ActionType.REVISE, text="42", visible_tokens=1),
        Action(ActionType.TERMINATE),
    ]

