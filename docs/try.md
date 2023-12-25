# Try builds

Instead of using your computer resources, you can enable Workflows in personal fork and then use try builds to test patches before sanding PR to servo.

## Triggering try runs

You can trigger try runs via:

- adding `T-` labels on PR (servo organization members only)
- dispatching workflows from personal fork
- running `mach try $try_string` command

`mach try` will send git `HEAD` (patches that are committed in current checkout) to try branch.

## Try strings

Try string can contain:

- `full`/`try` keyword that will be expanded to `linux mac windows`
- `fail-fast` marker keyword that will set [matrix fail-fast](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#jobsjob_idstrategyfail-fast) to true
- config tuples: `name[option=value, option2=value]` (`name` is also valid if `name` is preset)

### Options

- `os` (possible values: `linux` (default), `windows` or `mac`)
- `layout` (selects layout for wpt tests; possible values: `all`, `2020`, `2013`, `none`)
- `unit-tests` (default: `true`)
- `profile` (`release` (default), `debug`, `production`)

### Presets

- `linux` (does not run any wpt tests)
- `mac`
- `win` or `windows`
- `wpt` or `linux-wpt` (runs wpt tests for `both` layouts on linux)
- `webgpu` (runs WebGPU CTS on linux with layout2020 and production profile)
- `wpt-2013` or `linux-wpt-2013` (runs wpt tests on `2013` layout)
- `wpt-2020` or `linux-wpt-2020` (runs wpt tests on `2020` layout)
- `mac-wpt`
- `mac-wpt-2013`
- `mac-wpt-2020`

Using tuple config with presets you can override presets options.
