# Readme

Run from the root servo directory with:
```shell
uv run etc/blink-perf-test-runner/main.py SERVO_BINARY [--webdriver port] [--prepend name]
```
It will return a results.json in bencher bmf format.
Not every test currently produces an output.
The `--prepend` argument can be used to prepend e.g. a cargo `profile` name to the result keys.
This should be done when uploading to bencher, in order to distinguish measurements with different
cargo profiles.
