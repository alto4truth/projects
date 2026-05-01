# ABYSS Design

## Direction

ABYSS starts from a Qwen 72B-class checkpoint and trains full parameters using
replicated distributed NES. The model adds virtual latent tokens and a value head
for MCTS.

## Model

```text
Qwen backbone
├── lm_head
├── value_head
└── virtual token embeddings
```

Virtual tokens are tokenizer special tokens:

```text
<|abyss_latent_0|>
<|abyss_latent_1|>
...
```

The model can generate and consume them. User-facing inference strips them from
visible output and reward penalizes hidden-token usage.

## NES Protocol

All nodes start from identical `theta_k`.

For step `k`, rank `r` uses:

```text
seed = hash(run_id, step, floor(rank / 2))
sign = +1 for even ranks, -1 for odd ranks
candidate = theta_k + sign * sigma * epsilon(seed)
```

Each node publishes only:

```text
rank, seed, sign, fitness, diagnostics
```

The coordinator writes a canonical manifest. Every node reconstructs the same
noise tensors and applies the same update, producing identical `theta_{k+1}`
without checkpoint transfer in the hot path.

## Coordinator

The coordinator is not a learner. It:

- launches Modal evaluator jobs;
- waits for quorum;
- writes the canonical step manifest;
- launches update/checkpoint jobs;
- publishes `latest.json` after a successful checkpoint commit.

It never computes gradients and never owns a unique copy of the model weights.

## Checkpoint Safety

Weights must not be lost. Therefore:

- checkpoints are versioned by step;
- manifests are stored next to checkpoints;
- `latest.json` is updated only after checkpoint files exist;
- old checkpoints are not deleted automatically;
- failed steps do not advance `latest.json`;
- recovery reloads the latest accepted checkpoint and replays deterministic NES
  manifests if needed.

