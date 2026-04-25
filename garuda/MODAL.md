**Setup**

Use `uv`, not `pip`, for the fastest path on a clean machine:

```bash
./scripts/modal-setup.sh
```

If `modal profile current` prints your active profile, this repo is ready for the next step.
The important real check is that `uv tool run modal app list` does not return a token error.

**Run**

Distributed tune from the repo root:

```bash
ARTIFACT_PREFIX=first-real-run ./scripts/modal-tune.sh
```

The default model path is now neural:

```bash
MODEL_TYPE=neural ARTIFACT_PREFIX=first-real-run ./scripts/modal-tune.sh
```

Evaluate an existing manifest in parallel:

```bash
ARTIFACT_PREFIX=first-real-run ./scripts/modal-eval-manifest.sh modal-runs/first-real-run/generation-001.manifest.json
```

Override tuning parameters with environment variables:

```bash
ARTIFACT_PREFIX=large-run MODEL_TYPE=neural POPULATION_SIZE=32 GENERATIONS=8 ITERATIONS=10 MAX_PLIES=24 ./scripts/modal-tune.sh
```

**What It Does**

- `modal_app.py` packages the JS chess engine into a Modal image.
- `evaluate_candidate()` is the remote worker for one NES candidate.
- `distributed_tune()` is the local coordinator that plans generations locally and fans worker jobs out with Modal.
- `model_type` flows through the Modal run and now supports `neural` and `feature`
- run artifacts are written locally under `modal-runs/<artifact-prefix>/`
- worker result JSON is also written to the Modal volume `garuda-chess-runs` under `/mnt/runs/<artifact-prefix>/`
- `status.json` is updated locally and in the Modal volume as each generation progresses
- `scripts/modal-setup.sh` installs and authenticates Modal with `uv`
- `scripts/modal-tune.sh` launches a distributed tuning run
- `scripts/modal-eval-manifest.sh` fans out evaluation for an existing manifest

**Current Limitation**

This repo is now past the install blocker and the auth blocker.
The next bottleneck is cold image-build time because `modal_app.py` currently installs Debian `nodejs` and `npm` inside the image.
