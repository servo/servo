# Try Guide

Try runs allows to build and test changes on GitHub CI without requiring code to be reviewed and landed.

## Triggering try runs

You can trigger try runs via:

- adding `T-` labels on PR (servo organization members only)
- dispatching workflows from GitHub UI on personal fork
- running `mach try $try_string` command that will send git `HEAD` (patches that are committed in current checkout) to try branch on personal fork.

## Try strings

Try string can contain:

- `full`/`try` keyword that will be expanded to `linux mac windows`
- `fail-fast` marker keyword that will set [matrix fail-fast](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#jobsjob_idstrategyfail-fast) to true
- config tuples: `name[option=value, option2=value]` (`name` is also valid if `name` is preset)

### Options

- `os` (possible values: `linux` (default), `windows` or `mac`)
- `layout` (selects layout for wpt tests; possible values: `all`/`both`, `2020`, `2013`, `none`)
- `wpt` (additional arguments to be passed to `mach test-wpt` in CI; usually used for limiting testing scope)
- `unit-tests` (default: `true`)
- `profile` (`release` (default), `debug`, `production`)

### Presets

- `linux` (does not run any wpt tests)
- `mac`
- `win`/`windows`
- `wpt`/`linux-wpt` (runs wpt tests for `both` layouts on linux)
- `webgpu` (runs WebGPU CTS on linux)
- `wpt-2013` or `linux-wpt-2013` (runs wpt tests on `2013` layout)
- `wpt-2020` or `linux-wpt-2020` (runs wpt tests on `2020` layout)
- `mac-wpt` (runs wpt tests for `both` layouts on mac)
- `mac-wpt-2013`
- `mac-wpt-2020`

Using tuple config with presets you can override presets options.
