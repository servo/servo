# How to run scenarios
The directory has both scenarios that could be run and common files used by scenarios. To run a scenario:
```bash
uv run etc/ci/scenario/servo_test_open_page_servo_plot.py
```
## available arguments
- `--target-os` has `linux`, `macos` and `ohos` as current options
- `

or, if required, do
```bash
UV_PROJECT=etc/ci/scenario uv run --active etc/ci/scenario/servo_test_open_page_servo_plot.py
```

# How to develop scenarios
## Python type checking
Currently, due to a `pyproject.toml` in the root of servo, the default `./mach test-tidy` does not fully check current directory (`etc/ci/scenario`).
It is crucial to currently manually check the Python scripts in this directory using
```bash
uv run --active pyrefly check etc/ci/scenario/
```

# Adding scenarios to CI
## Mitmproxy
Mitmproxy currently requires `CI=1` env to run
## Target binary
Current default binary is defined in the `common_function_for_servo_test.py` as
```python
DEFAULT_SERVO_BIN_PATH = "./target/release/servoshell"
```
But could be overriden by passing `--servo-bin` arg to the scenario

# Using memory_usage_plotter
There are three intended uses for the `memory_usage_plotter.py`
1. It is included in the scenarios and can log the memory (to the `.csv`) when running the scenario or even plot in place without leaving `.csv`.
1. Can be used to convert already existing `.csv` into a rendered plot image.
1. To track the memory of already running servo process without explicit scenario `operator()`.

## Notable options or arguments
The script has a `MemoryLoggingOptions` that could be imported and used to set all the options.
```Python
memory_logging_options = MemoryLoggingOptions(
    log_to_file=True, plot=True, pre_time=2, post_time=5, verbose=True, reset_tab=True
)
```
or when the script is run in a standalone mode you can do `-h` to see whats there
```bash
UV_PROJECT=etc/ci/scenario uv run --active etc/ci/scenario/memory_usage_plotter.py -h
```

# Running blink-perf-test
There is actually another runner of blink-perf-test present in the codebase, but I guess the current one is more general and might be better because it hadles more issues. Thus, this `blink-perf-test` is primarely made for running on OHOS.
But as this `blink-perf-test` is now also has access to the memory loggin, the scope of it also now includes `linux` and `macos`
So, as usual, to run:
```bash
UV_PROJECT=etc/ci/scenario uv run --active etc/ci/scenario/blink_perf_test.py -h
```

To check if works, there is a `single-test` mode, and extra `-mve` would also add more info to the output.
```bash
UV_PROJECT=etc/ci/scenario uv run --active etc/ci/scenario/blink_perf_test.py -mves
```
Will produce both `results.json` and `blink_perf_test_logs.csv` that would include bench of the run and memory info.