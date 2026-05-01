# ABYSS LLM Design

This document is the current design agreement for ABYSS LLM.

## Goal

Build a self-hosted 72B-class LLM system trained with distributed NES and
MCTS-style episode search. The system starts from a top open-weight Qwen
checkpoint, adds hidden virtual tokens and a value head, and runs on donated
Modal B200 capacity.

## Base Model

Default foundation:

```text
Qwen/Qwen2.5-72B
```

Use a base model, not an instruct model, unless the goal changes to immediate
assistant behavior over further training.

## Hardware

Primary backend:

```text
Modal B200
```

Default node:

```text
1 NES node = 8x B200
```

Default population:

```text
population_size = number_of_nodes
```

Example:

```text
16 nodes * 8 B200 = 128 B200 per NES step
```

## Precision

Recommended for B200:

```text
canonical weights/checkpoints: bf16
forward activations: bf16
value/loss/fitness scalar math: fp32
NES update accumulation: fp32 blockwise
saved checkpoint: bf16 safetensors
```

Do not require full fp32 `theta`; 72B fp32 is about 288 GB.

For single-node 72B probing, use FP8 quantization first:

```text
Transformers FineGrainedFP8Config
weights/activations: FP8 where supported
value head: small auxiliary head
reward model: disabled or separate until base path is proven
```

## Model Architecture

```text
Qwen backbone
├── lm_head        -> policy over tokens/actions
├── value_head     -> expected net episode reward
└── virtual token embeddings
```

The value head is required because MCTS needs state value estimates.

## Virtual Tokens

Virtual tokens are added as tokenizer special tokens:

```text
<|abyss_latent_0|>
<|abyss_latent_1|>
...
<|abyss_latent_N|>
```

They are visible to the model and hidden from the user. They are used for hidden
reasoning/scratchpad state.

Every virtual token has a reward cost:

```text
vt_cost = 0.00001
```

Episode reward includes:

```text
reward =
  task_score
  - virtual_token_count * vt_cost
  - visible_token_count * visible_cost
  - tool_calls * tool_cost
  - search_time * time_cost
  - failed_verifier_penalty
```

The value head should learn expected net reward, not merely probability of
correctness.

## Reward Model Terminal Judge

Until domain verifiers cover every task, ABYSS can use an external reward model
as the terminal judge for an episode:

```text
rm_score = reward_model(prompt, final_answer)
final_reward =
  rm_score * reward_model_weight
  - virtual_token_count * vt_cost
  - visible_token_count * visible_cost
  - tool_calls * tool_cost
  - search_time * time_cost
```

The virtual-token penalty is always separate from the reward model score. Reward
models are useful for general answer quality, but exact verifiers should override
or supplement them for math, code, chess, and other checkable domains.

Current initial reward is intentionally simple:

```text
reward = reward_model(prompt, final_answer) - virtual_token_count * 0.00001
```

The value head is trained against this same net terminal reward. With NES, that
means candidate fitness includes a value calibration term:

```text
fitness = mean(final_reward) - value_loss_weight * mse(value_head(state), final_reward)
```

This makes the value head learn to approximate the reward model's terminal
judgement after cost penalties, rather than remaining an unused auxiliary head.

Prompt tasks can come from built-in prompts, local JSONL, or Hugging Face
datasets such as `tatsu-lab/alpaca`.

## MCTS As Episode Game

One answer is one episode.

State:

```text
prompt
+ hidden virtual-token reasoning
+ visible draft
+ tool/verifier results
+ rolling context/state summary
```

Actions are macro-actions, not single-token actions:

```text
emit hidden reasoning chunk
emit visible answer chunk
call verifier/tool
revise/branch
summarize state
terminate
```

The LLM acts like a game policy:

```text
policy(state) -> candidate next actions
value(state)  -> expected net final reward
```

MCTS searches over these macro-actions. Terminal reward is computed when the
episode ends through verifiers, tests, game outcome, or task scoring.

## Stopping Rule

Stopping certainty should come from MCTS visits, not self-reported confidence.

Preferred certainty:

```text
certainty = visits(top1) / (visits(top1) + visits(top2))
```

Stop when:

```text
root_visits >= min_visits
and certainty >= 0.95
```

Also stop if further search/reasoning is not worth its cost:

```text
expected_gain_from_continue <= expected_extra_cost
```

`TERMINATE` should be a normal MCTS action.

## NES Training Protocol

There is no central learner that owns unique weights.

All nodes start from the same `theta_k`, where:

```text
theta = all model parameters
```

That includes:

```text
Qwen weights
lm_head
value_head
virtual token embeddings
```

For NES step `k`, rank `r`:

```text
pair = floor(r / 2)
seed = hash(run_id, step, pair)
sign = +1 for even ranks
sign = -1 for odd ranks
epsilon = deterministic_noise(seed)
candidate = theta_k + sign * sigma * epsilon
fitness = evaluate(candidate)
```

Each node publishes only compact scalar metadata:

```json
{
  "run_id": "abyss-72b-nes-001",
  "step": 123,
  "rank": 7,
  "seed": 918273645,
  "sign": -1,
  "fitness": 0.7132,
  "diagnostics": {}
}
```

The coordinator writes a canonical manifest:

```json
{
  "run_id": "abyss-72b-nes-001",
  "step": 123,
  "sigma": 0.0005,
  "learning_rate": 0.02,
  "participants": []
}
```

Every node reads the same manifest, reconstructs all perturbations by seed, and
applies the same update:

```text
theta_{k+1} = theta_k + NES_update(manifest)
```

No weights are transferred during the hot path. Communication is seeds,
fitnesses, diagnostics, and manifests.

## Antithetic Sampling

NES should use antithetic pairs:

```text
theta + epsilon
theta - epsilon
```

This reduces variance. Population size should be even.

## Coordinator

The coordinator can run on Modal CPU.

It is not a learner. It:

- launches one evaluator job per population member;
- waits for quorum;
- collects fitness JSON files;
- writes the canonical manifest;
- launches update/checkpoint jobs;
- publishes `latest.json` only after checkpoint commit.

It does not update weights.

## Checkpoint Safety

Weights must not be lost.

Checkpoint layout:

```text
/mnt/abyss/checkpoints/step-00000000/
/mnt/abyss/checkpoints/step-00000001/
/mnt/abyss/manifests/step-00000000.json
/mnt/abyss/latest.json
```

Rules:

- old checkpoints are not deleted automatically;
- failed steps do not advance `latest.json`;
- `latest.json` is written atomically via temp-file replace;
- new checkpoint directories include an `ABYSS_COMMITTED` marker;
- `value_head.pt` is saved next to the Hugging Face base model files;
- checkpoint directories include `manifest.json`, `checkpoint_meta.json`, and
  `publish_meta.json`;
- `latest.json` includes a checkpoint fingerprint;
- periodic checksums should detect node drift;
- if drift occurs, reload the canonical checkpoint.

Even with identical B200 machines, floating-point nondeterminism can happen, so
rank 0 or another reference node should publish canonical checkpoints.

## Training Performance

Training objective is maximum useful fitness evaluations per hour.

Important tactics:

- keep Modal workers long-lived when possible;
- avoid reloading 72B per candidate;
- batch MCTS rollouts;
- reuse KV cache where possible;
- use bf16 for model compute;
- use fp32 blockwise update accumulation;
- write checkpoints at safe generation boundaries, not every micro-action.

Modal spending rule:

```text
CPU mock smoke: allowed freely for protocol checks
single small-GPU probe: allowed only for model-load/runtime checks
8xB200 node: use only for meaningful 72B candidate evaluation
full population: use only after fitness/MCTS code produces useful signal
```

## Inference Performance

Inference should be optimized separately from training.

Likely serving backend:

```text
vLLM or SGLang
```

Inference server requirements:

- tensor parallel across B200s;
- continuous batching;
- KV cache;
- hidden virtual-token filtering;
- optional MCTS for hard prompts;
- track hidden-token count and cost.

## Non-Goals

- No LoRA as the main training path.
- No toy Transformer.
- No token-level MCTS over the full vocabulary.
- No central learner that sends full checkpoints every step.
- No naive NES that regenerates different noise during update.
- No self-RLAIF without verifiers/judges/gates.
