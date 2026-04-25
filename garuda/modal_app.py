from __future__ import annotations

import json
import subprocess
from pathlib import Path
from typing import Any

import modal


APP_NAME = "garuda-chess"
REPO_ROOT = Path("/root/garuda")
RUNS_DIR = Path("/mnt/runs")
DEFAULT_VOLUME_NAME = "garuda-chess-runs"


image = (
    modal.Image.from_registry("node:20-bookworm-slim", add_python="3.11")
    .add_local_file("package.json", f"{REPO_ROOT}/package.json", copy=True)
    .add_local_file("package-lock.json", f"{REPO_ROOT}/package-lock.json", copy=True)
    .add_local_dir(
        "js",
        f"{REPO_ROOT}/js",
        copy=True,
        ignore=["**/node_modules", "**/__pycache__", "**/*.pyc"],
    )
    .run_commands(
        f"cd {REPO_ROOT} && npm install --omit=dev --ignore-scripts",
    )
    .workdir(str(REPO_ROOT))
)

app = modal.App(APP_NAME)
runs_volume = modal.Volume.from_name(DEFAULT_VOLUME_NAME, create_if_missing=True, version=2)


def _run_cli(args: list[str]) -> str:
    command = ["node", "js/src/chess/cli.js", *args]
    completed = subprocess.run(
        command,
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout


def _write_local_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n")


def _emit_volume_artifact(prefix: str | None, name: str, payload: str) -> str | None:
    if not prefix:
        return None
    target_dir = RUNS_DIR / prefix
    target_dir.mkdir(parents=True, exist_ok=True)
    target_path = target_dir / name
    target_path.write_text(payload)
    runs_volume.commit()
    return str(target_path)


def _write_status(prefix: str, run_dir: Path, payload: dict[str, Any]) -> None:
    _write_local_json(run_dir / "status.json", payload)
    _emit_volume_artifact(prefix, "status.json", json.dumps(payload, indent=2) + "\n")


@app.function(
    image=image,
    timeout=60 * 60,
    startup_timeout=60 * 20,
    cpu=2,
    volumes={str(RUNS_DIR): runs_volume},
)
def evaluate_candidate(
    task_json: str,
    iterations: int = 6,
    max_plies: int = 16,
    cpuct: float = 1.35,
    fitness: str = "mixed",
    model_type: str = "feature",
    artifact_prefix: str = "",
) -> str:
    payload = _run_cli(
        [
            "dist:worker",
            "--task",
            task_json,
            "--iterations",
            str(iterations),
            "--maxPlies",
            str(max_plies),
            "--cpuct",
            str(cpuct),
            "--fitness",
            fitness,
            "--modelType",
            model_type,
        ]
    )
    task = json.loads(task_json)
    _emit_volume_artifact(artifact_prefix or None, f"{task['id']}.result.json", payload)
    return payload


@app.function(
    image=image,
    timeout=60 * 60,
    startup_timeout=60 * 20,
    cpu=2,
    volumes={str(RUNS_DIR): runs_volume},
)
def run_remote_cli(command: str, argv_json: str = "[]", artifact_prefix: str = "") -> str:
    argv = json.loads(argv_json)
    if not isinstance(argv, list):
        raise ValueError("argv_json must decode to a JSON array")
    payload = _run_cli([command, *[str(item) for item in argv]])
    safe_name = command.replace(":", "_")
    _emit_volume_artifact(artifact_prefix or None, f"{safe_name}.json", payload)
    return payload


@app.local_entrypoint()
def distributed_tune(
    population_size: int = 16,
    generations: int = 4,
    iterations: int = 6,
    max_plies: int = 16,
    cpuct: float = 1.35,
    sigma: float = 0.12,
    learning_rate: float = 0.18,
    seed: int = 1337,
    fitness: str = "mixed",
    model_type: str = "neural",
    artifact_prefix: str = "latest",
) -> None:
    vector_payload = json.loads(_run_cli(["vector", "--modelType", model_type]))
    center = vector_payload["vector"]
    run_dir = Path("modal-runs") / artifact_prefix
    run_dir.mkdir(parents=True, exist_ok=True)
    _write_status(
        artifact_prefix,
        run_dir,
        {
            "phase": "initializing",
            "artifactPrefix": artifact_prefix,
            "modelType": model_type,
            "populationSize": population_size,
            "generations": generations,
            "iterations": iterations,
            "maxPlies": max_plies,
            "cpuct": cpuct,
            "fitness": fitness,
        },
    )

    aggregated_history = []
    baseline = json.loads(
        _run_cli(
            [
                "eval",
                "--modelType",
                model_type,
                "--vector",
                json.dumps(center),
                "--iterations",
                str(iterations),
                "--maxPlies",
                str(max_plies),
                "--cpuct",
                str(cpuct),
            ]
        )
    )

    for generation in range(1, generations + 1):
        print(f"[modal] generation {generation}/{generations}: planning {population_size} candidates", flush=True)
        _write_status(
            artifact_prefix,
            run_dir,
            {
                "phase": "planning",
                "generation": generation,
                "generations": generations,
                "artifactPrefix": artifact_prefix,
                "modelType": model_type,
                "populationSize": population_size,
                "fitness": fitness,
            },
        )
        manifest = json.loads(
            _run_cli(
                [
                    "dist:plan",
                    "--modelType",
                    model_type,
                    "--generation",
                    str(generation),
                    "--fitness",
                    fitness,
                    "--populationSize",
                    str(population_size),
                    "--sigma",
                    str(sigma),
                    "--learningRate",
                    str(learning_rate),
                    "--seed",
                    str(seed + generation - 1),
                    "--center",
                    json.dumps(center),
                ]
            )
        )
        _write_local_json(run_dir / f"generation-{generation:03d}.manifest.json", manifest)

        task_payloads = [json.dumps(task) for task in manifest["tasks"]]
        print(f"[modal] generation {generation}/{generations}: evaluating {len(task_payloads)} candidates", flush=True)
        _write_status(
            artifact_prefix,
            run_dir,
            {
                "phase": "evaluating",
                "generation": generation,
                "generations": generations,
                "artifactPrefix": artifact_prefix,
                "modelType": model_type,
                "taskCount": len(task_payloads),
                "fitness": fitness,
            },
        )
        results = [
            json.loads(payload)
            for payload in evaluate_candidate.map(
                task_payloads,
                kwargs={
                    "iterations": iterations,
                    "max_plies": max_plies,
                    "cpuct": cpuct,
                    "fitness": fitness,
                    "model_type": model_type,
                    "artifact_prefix": artifact_prefix,
                },
            )
        ]

        _write_local_json(run_dir / f"generation-{generation:03d}.results.json", results)

        print(f"[modal] generation {generation}/{generations}: aggregating", flush=True)
        summary = json.loads(
            _run_cli(
                [
                    "dist:aggregate",
                    "--modelType",
                    model_type,
                    "--manifest",
                    json.dumps(manifest),
                    "--results",
                    json.dumps(results),
                    "--iterations",
                    str(iterations),
                    "--maxPlies",
                    str(max_plies),
                    "--cpuct",
                    str(cpuct),
                    "--fitness",
                    fitness,
                ]
            )
        )
        _write_local_json(run_dir / f"generation-{generation:03d}.summary.json", summary)
        _write_status(
            artifact_prefix,
            run_dir,
            {
                "phase": "generation_complete",
                "generation": generation,
                "generations": generations,
                "artifactPrefix": artifact_prefix,
                "modelType": model_type,
                "bestScore": summary["bestScore"],
                "nextCenterScore": summary["nextCenterScore"],
                "bestTaskId": summary["bestTaskId"],
                "completedTaskCount": summary["completedTaskCount"],
            },
        )
        print(
            f"[modal] generation {generation}/{generations}: best={summary['bestScore']:.4f} center={summary['nextCenterScore']:.4f}",
            flush=True,
        )

        center = summary["nextCenter"]
        aggregated_history.append(summary["historyEntry"])

    final_payload = {
        "kind": "modal-distributed-run",
        "modelType": model_type,
        "fitness": fitness,
        "populationSize": population_size,
        "generations": generations,
        "iterations": iterations,
        "maxPlies": max_plies,
        "cpuct": cpuct,
        "baseline": baseline,
        "finalCenter": center,
        "history": aggregated_history,
        "artifactPrefix": artifact_prefix,
        "localRunDir": str(run_dir),
        "remoteVolumeDir": f"{RUNS_DIR}/{artifact_prefix}",
    }
    _write_local_json(run_dir / "final.json", final_payload)
    _write_status(
        artifact_prefix,
        run_dir,
        {
            "phase": "completed",
            "artifactPrefix": artifact_prefix,
            "modelType": model_type,
            "bestScore": max((entry["bestScore"] for entry in aggregated_history), default=baseline.get("mixed", 0)),
            "finalCenter": center,
        },
    )
    print(json.dumps(final_payload, indent=2))


@app.local_entrypoint()
def eval_generation(
    manifest_path: str,
    iterations: int = 6,
    max_plies: int = 16,
    cpuct: float = 1.35,
    fitness: str = "mixed",
    model_type: str = "neural",
    artifact_prefix: str = "manual",
) -> None:
    manifest = json.loads(Path(manifest_path).read_text())
    task_payloads = [json.dumps(task) for task in manifest["tasks"]]
    results = [
        json.loads(payload)
        for payload in evaluate_candidate.map(
            task_payloads,
            kwargs={
                "iterations": iterations,
                "max_plies": max_plies,
                "cpuct": cpuct,
                "fitness": fitness,
                "model_type": model_type,
                "artifact_prefix": artifact_prefix,
            },
        )
    ]
    print(json.dumps(results, indent=2))
