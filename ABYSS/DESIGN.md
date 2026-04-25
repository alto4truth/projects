# ABYSS Design Decisions

## Why NES?

### Problem: AdamW doesn't scale across geographies
- AdamW requires gradient synchronization
- Cross-region bandwidth is insufficient
- Need compression, added complexity

### Solution: NES (Natural Evolution Strategies)
- **No gradients** — only fitness values
- **O(1) communication** — just scalar rankings
- **Minimal state** — only model weights

## Architecture

### Memory: 70B model on 1× B200 (192GB)
- Model: 70B × FP16 = 140GB ✓
- NES state: `theta` (current weights) = 140GB ✓
- **Total: 140GB** < 192GB ✓

### No AdamW state!
- No momentum (m)
- No variance (v)
- **50% memory savings**

### NES workflow
```
1. Generate noise N(0,1) on-the-fly
2. candidate = theta + noise × sigma
3. Evaluate fitness
4. Discard noise
5. Aggregate rankings across GPUs
6. Update theta
```

### Communication
- Only fitness rankings (scalars!)
- No gradients transferred
- ~few KB per step vs ~28GB with AdamW

## Configuration

```python
{
    "model_size": "70B",
    "precision": "fp16",
    "optimizer": "NES",
    "popsize": 256,
    "sigma": 0.01,
    "learning_rate": 0.01,
    "distributed": "all_reduce_rankings"
}
```

## Hardware Requirements

| Model | GPUs | Memory |
|-------|------|--------|
| 70B   | 1× B200 | 140GB |
| 70B   | 8× B200 | Full sharding |

## References
- NES: https://arxiv.org/abs/1106.4487
- OpenAI ES: https://blog.openai.com/evolution-strategies/