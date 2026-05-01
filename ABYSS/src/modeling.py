from __future__ import annotations

from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True)
class AbyssSpecialTokens:
    latent_tokens: list[str]

    @property
    def latent_token_set(self) -> set[str]:
        return set(self.latent_tokens)


def build_latent_tokens(cfg: dict[str, Any]) -> list[str]:
    return [
        f"{cfg['prefix']}{idx}{cfg['suffix']}"
        for idx in range(int(cfg["count"]))
    ]


def torch_dtype(name: str):
    import torch

    mapping = {
        "float32": torch.float32,
        "float16": torch.float16,
        "bfloat16": torch.bfloat16,
        "bf16": torch.bfloat16,
    }
    return mapping[str(name)]


def load_abyss_model(cfg: dict[str, Any], model_source: str | None = None):
    import torch
    import torch.nn as nn
    from transformers import AutoModelForCausalLM, AutoTokenizer

    source = model_source or cfg["model_name"]
    print(f"[abyss] loading tokenizer: {source}", flush=True)
    tokenizer = AutoTokenizer.from_pretrained(
        source,
        trust_remote_code=bool(cfg["model"].get("trust_remote_code", True)),
    )
    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    quantization_config = build_quantization_config(cfg)
    print(f"[abyss] loading base model: {source}", flush=True)
    base = AutoModelForCausalLM.from_pretrained(
        source,
        torch_dtype=None if quantization_config is not None else torch_dtype(cfg["model"].get("torch_dtype", "bfloat16")),
        trust_remote_code=bool(cfg["model"].get("trust_remote_code", True)),
        low_cpu_mem_usage=True,
        device_map=cfg["model"].get("device_map"),
        quantization_config=quantization_config,
    )
    latent_tokens = build_latent_tokens(cfg["virtual_tokens"])
    added = tokenizer.add_special_tokens({"additional_special_tokens": latent_tokens})
    if added:
        print(f"[abyss] resizing embeddings, added_tokens={added}", flush=True)
        base.resize_token_embeddings(len(tokenizer))
    print("[abyss] attaching value head", flush=True)
    model = AbyssForCausalLMWithValue(base, float(cfg["model"].get("value_head_init_std", 0.02)))
    load_value_head_if_present(model, source)
    print("[abyss] model ready", flush=True)
    return model, tokenizer, AbyssSpecialTokens(latent_tokens)


class AbyssForCausalLMWithValue:
    def __new__(cls, base_model, value_head_init_std: float):
        import torch.nn as nn

        class Wrapped(nn.Module):
            def __init__(self):
                super().__init__()
                self.base_model = base_model
                hidden_size = base_model.config.hidden_size
                self.value_head = nn.Linear(hidden_size, 1, bias=False)
                nn.init.normal_(self.value_head.weight, mean=0.0, std=value_head_init_std)

            def forward(self, input_ids=None, attention_mask=None, labels=None, **kwargs):
                outputs = self.base_model(
                    input_ids=input_ids,
                    attention_mask=attention_mask,
                    labels=labels,
                    output_hidden_states=True,
                    **kwargs,
                )
                last_hidden = outputs.hidden_states[-1]
                values = self.value_head(last_hidden).squeeze(-1)
                outputs.values = values
                return outputs

            def generate(self, *args, **kwargs):
                return self.base_model.generate(*args, **kwargs)

            @property
            def config(self):
                return self.base_model.config

        return Wrapped()


def save_abyss_model(model, tokenizer, save_dir: str | Any) -> None:
    import torch
    from pathlib import Path

    save_path = Path(save_dir)
    save_path.mkdir(parents=True, exist_ok=True)
    model.base_model.save_pretrained(str(save_path))
    tokenizer.save_pretrained(str(save_path))
    torch.save(model.value_head.state_dict(), save_path / "value_head.pt")


def load_value_head_if_present(model, model_source: str) -> None:
    from pathlib import Path
    import torch

    path = Path(model_source) / "value_head.pt"
    if path.exists():
        state = torch.load(path, map_location="cpu")
        model.value_head.load_state_dict(state)


def build_quantization_config(cfg: dict[str, Any]):
    quantization = cfg["model"].get("quantization")
    if quantization in {None, "none", False}:
        return None
    if quantization == "fp8":
        from transformers import FineGrainedFP8Config

        print("[abyss] using FineGrainedFP8Config", flush=True)
        return FineGrainedFP8Config()
    raise ValueError(f"Unsupported quantization: {quantization}")
