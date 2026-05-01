from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from modeling import torch_dtype


@dataclass
class RewardModel:
    model: Any
    tokenizer: Any
    device: str

    def score(self, prompt: str, answer: str) -> float:
        import torch

        messages = [
            {"role": "user", "content": prompt},
            {"role": "assistant", "content": answer},
        ]
        if hasattr(self.tokenizer, "apply_chat_template"):
            text = self.tokenizer.apply_chat_template(
                messages,
                tokenize=False,
                add_generation_prompt=False,
            )
        else:
            text = f"User: {prompt}\nAssistant: {answer}"
        encoded = self.tokenizer(text, return_tensors="pt", truncation=True)
        encoded = {key: value.to(self.device) for key, value in encoded.items()}
        with torch.no_grad():
            outputs = self.model(**encoded)
            logits = getattr(outputs, "logits", None)
            if logits is None:
                score = outputs[0]
            else:
                score = logits
            return float(score.reshape(-1)[-1].float().item())


def load_reward_model(cfg: dict[str, Any]) -> RewardModel | None:
    rm_cfg = cfg.get("reward_model", {})
    if not rm_cfg.get("enabled", False):
        return None

    from transformers import AutoModelForSequenceClassification, AutoTokenizer

    device = str(rm_cfg.get("device", "cuda"))
    print(f"[abyss] loading reward tokenizer: {rm_cfg['name']}", flush=True)
    tokenizer = AutoTokenizer.from_pretrained(
        rm_cfg["name"],
        trust_remote_code=True,
    )
    print(f"[abyss] loading reward model: {rm_cfg['name']}", flush=True)
    model = AutoModelForSequenceClassification.from_pretrained(
        rm_cfg["name"],
        torch_dtype=torch_dtype(rm_cfg.get("torch_dtype", "bfloat16")),
        trust_remote_code=True,
    )
    model.to(device)
    model.eval()
    print("[abyss] reward model ready", flush=True)
    return RewardModel(model=model, tokenizer=tokenizer, device=device)


def normalize_reward_model_score(score: float) -> float:
    # Keep first version deliberately simple. Later replace with calibration per RM.
    return score
