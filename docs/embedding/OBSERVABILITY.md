# Embedding API Observability

W5.6 asks for four signals on every `embedding-api.yml` run, so that regressions
show up as a diff in CI instead of being discovered in a release
(`ROADMAP.md` Phase 5). This document records what is measured, where it comes
from, and why it is reported rather than enforced.

## The metrics

| Metric | Plan signal | What it measures | How |
| --- | --- | --- | --- |
| `startup_ms` | startup wall-time | `servo_builder_build()` — building the engine itself (constellation, compositor, resource manager, ...) | Timed in `run_c_integration_tests()`, in a fresh process |
| `load_complete_ms` | throughput | `servo_webview_builder_build()` → first page reaching `SERVO_LOAD_STATUS_COMPLETE`, for a fixed `data:` page | Timed in `test_webview_load_and_screenshot` |
| `screenshot_ms` | throughput | `servo_webview_paint()` plus `servo_webview_take_screenshot()` request → the raster callback, i.e. paint plus readback | Timed around `on_new_frame_ready` / `on_screenshot` |
| `js_ms` | throughput | A fixed JavaScript loop (100000 iterations, the same script everywhere), request → result callback | `servo_webview_evaluate_javascript_v2` with `JS_WORKLOAD` |
| `cdylib_bytes` | artifact size | The size of the built `libservo_capi.so` / `.dylib` / `servo_capi.dll` | Measured by the CI step, which alone can see the build output |
| C API job result | crash proxy | Whether the `capi` job passed. On hosted runners a crash surfaces as a failed job and nothing else; there is no core-dump service to count | `needs.capi.result`, recorded by the report job |

Every number is recorded from real work done by the existing test run: the
metrics cost no extra build, no extra process, and no extra test time. `-1` /
absent means "not measured" and is reported as `-`, so a metric that stops being
collected shows up as a gap instead of a silent zero.

Two of the five have a resolution worth stating:

- `screenshot_ms` is measured in whole milliseconds and the raster callback is
  delivered synchronously by `servo_webview_take_screenshot()`, so it reads `0`
  when the paint is fast. The first version timed only the call itself, which
  *always* read `0`; the timer now starts before `servo_webview_paint()`, so the
  number covers what the label claims and would only become visible if the paint
  path slowed down by a millisecond or more.
- `cdylib_bytes` is the unstripped debug artifact the gate builds, which is why
  it is hundreds of megabytes. It is compared against itself, not against a
  release artifact.

## How it flows

1. `tests/capi/c/test_integration.c` keeps the timings in `g_metric_*` globals and
   writes them as one line of JSON to the file named by the
   `SERVO_OBSERVABILITY_FILE` environment variable, which `embedding-api.yml`
   sets to an absolute path in the workspace. Without the variable (a plain
   `cargo test` run) the numbers are only logged.
2. The `Collect observability metrics` step in the `capi` job adds
   `cdylib_bytes` and the platform name to that JSON and uploads it as the
   `observability-metrics-<platform>` artifact. It runs even when the tests
   failed, because a failing run is part of the crash proxy.
3. The `Observability report` job downloads all three artifacts and runs
   `etc/ci/embedding_observability.py`, which walks the download directory
   recursively (each artifact lands in a directory of its own), writes a table
   to the run summary plus an `observability-summary` artifact, and diffs the
   numbers against `observability_baseline.json`.

The first run of this job found *no* metric files and reported success while
measuring nothing, because the script looked only one level deep in the
download directory. The layout is now walked recursively, and the job fails when
the metrics are missing although the `capi` job succeeded and has no other
explanation — a green observability job that measured nothing is worse than a
red one. The reporter still reports rather than fails on a *measured*
regression, and a missing metric file from a platform whose job failed is
reported (that platform's column is simply absent from the table).

Run `39310767848` is the first run whose report read the committed baseline and
printed the change table, so the "metrics are collected and diffed" path is
CI-verified rather than only locally exercised.

## Why `mach smoketest` is not used for startup

The plan suggested `mach smoketest` startup wall-time. That cannot run here:
`mach smoketest` measures `servoshell`, so the job would have to build the shell
app in addition to the C API (a multi-hour build), and it needs a display on the
hosted runners. The embedding gate is about the C API, and `servo_builder_build`
plus a fixed page load is the closest equivalent that the gate can already
afford. It is a different quantity from shell startup, and is only compared
against itself.

## Why this reports instead of gating

Absolute timings on GitHub-hosted runners move by tens of percent between runs
on the same commit: the runners are shared, their CPU models differ within a
matrix, and the cache state differs. A hard threshold would therefore be noise
that fails builds, which is exactly what a gate must not be.

- The baseline file records the first values that were actually measured, with
  the run that produced them.
- The diff table flags a change larger than 25% with `(!)`. That flag is
  informational; the job still exits 0.
- The job *does* exit non-zero when the input is malformed, because that means
  the collection broke rather than the numbers moving, and when no metrics are
  found at all although the `capi` job succeeded.

Turning any of these into a real gate needs a quieter measurement first:
dedicated self-hosted runners with medians over several samples (what
`bencher.yml` does for file size and Speedometer), or a much larger threshold
chosen from measured noise. `bencher.yml` cannot carry these metrics as-is: it
is a reusable workflow driven by the try/benchmark flows, it requires
`BENCHER_API_TOKEN`, and it selects self-hosted bench runners.

## Updating the baseline

`docs/embedding/observability_baseline.json` has this shape:

```json
{
  "recorded_at": "2026-10-20",
  "source_run": 1234567890,
  "source_sha": "0123456789abcdef",
  "platforms": {
    "Linux": {
      "startup_ms": 0, "load_complete_ms": 0, "screenshot_ms": 0, "js_ms": 0,
      "cdylib_bytes": 0
    }
  }
}
```

The committed baseline comes from run `39307687792` (all three platforms green).
It has no `screenshot_ms` entry on purpose: that run measured the metric before
the timing fix described above, so its value was always `0` and would be a
baseline for a different measurement. The next run after that fix establishes
the first real value; a missing entry, like a zero one, is reported as `-` rather
than as a percentage change.

To refresh it, take the values from the `Observability report` table of a run
you trust, copy them in, and update `recorded_at` / `source_run` / `source_sha`
to that run. There is no automatic update: CI cannot push, and a baseline that
silently rewrites itself hides exactly the drift it exists to show.
