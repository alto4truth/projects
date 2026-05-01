# ABYSS LLM

Replicated distributed NES infrastructure for ABYSS on Modal B200.

Each node keeps the same Qwen checkpoint locally, evaluates one perturbation,
exchanges only scalar fitness records through a Modal Volume, then applies the
same deterministic NES update from a canonical manifest.

There is no LoRA path, no toy Transformer path, and no central learner that owns
the weights.

## Install

```bash
pip install torch transformers datasets accelerate pyyaml
```

## Smoke Check

```bash
python src/train.py dry-run --config config/abyss_smoke_nes.yaml
python scripts/local_nes_smoke.py
```

## Modal Generation

```bash
modal run app.py --config-path config/abyss72b_nes.yaml --steps 1
```

The Modal coordinator runs on CPU. It launches one `B200:8` evaluator per
population member, writes a canonical step manifest, then launches update jobs.
The coordinator never updates or transfers model weights.

## Checkpoint Safety

Checkpoints live in the `abyss-nes-state` Modal Volume:

```text
/mnt/abyss/checkpoints/step-00000000/
/mnt/abyss/checkpoints/step-00000001/
/mnt/abyss/latest.json
```

`latest.json` is updated only after the new checkpoint directory and manifest
metadata are written and the volume commit succeeds.
