Servo Page Load Time Test
==============

# Prerequisites

* Python3

# Basic Usage

`./mach test-perf` can be used to run a performance test on your servo build. The test result JSON will be saved to `etc/ci/performance/output/`. You can then run `python test_differ.py` to compare these two test results. Run `python test_differ.py -h` for instructions.

# Setup for CI machine
## CI for Servo

* Setup your Treeherder client ID and secret as environment variables `TREEHERDER_CLIENT_ID` and `TREEHERDER_CLIENT_SECRET`
* Run `./mach test-perf --submit` to run and submit the result to Perfherder.

## CI for Gecko

* Install Firefox Nightly in your PATH
* Download [geckodriver](https://github.com/mozilla/geckodriver/releases) and add it to the `PATH` (e.g. for Linux `export PATH=$PATH:path/to/geckodriver`)
* `export FIREFOX_BIN=/path/to/firefox`
* `pip install selenium`
* Run `python gecko_driver.py` to test
* Run `test_all.sh --gecko --submit` (omit `--submit` if you don't want to submit to perfherder)

# How it works

* The testcase is from tp5, every testcase will run 20 times, and we take the median.
* Some of the tests will make Servo run forever, it's disabled right now. See https://github.com/servo/servo/issues/11087
* Each testcase is a subtest on Perfherder, and their summary time is the geometric mean of all the subtests.
* Notice that the test is different from the Talos TP5 test we run for Gecko. So you can NOT conclude that Servo is "faster" or "slower" than Gecko from this test.

# Comparing the performance before and after a patch

* Run the test once before you apply a patch, and once after you apply it.
* `python test_differ.py output/perf-<before time>.json output/perf-<after time>.json`
* Green lines means loading time decreased, Blue lines means loading time increased.

# Add your own test

* Add you test case (html file) to the `page_load_test/` folder. For example we can create a `page_load_test/example/example.html`
* Add a manifest (or modify existing ones) named `page_load_test/example.manifest`
* Add the lines like this to the manifest:

```
http://localhost:8000/page_load_test/example/example.html
# This is a comment
# Pages got served on a local server at localhost:8000
```
* Modify the `MANIFEST=...` link in `test_all.sh` and point that to the new manifest file.

# Unit tests

You can run all unit tests (include 3rd-party libraries) with `python -m pytest`.

Individual test can be run by `python -m pytest <filename>`:

* `test_runner.py`
* `test_submit_to_perfherder.py`

# Advanced Usage

## Test Perfherder Locally

If you want to test the data submission code in `submit_to_perfherder.py` without getting a credential for the production server, you can setup a local treeherder VM. If you don't need to test `submit_to_perfherder.py`, you can skip this step.

* Add `192.168.33.10    local.treeherder.mozilla.org` to `/etc/hosts`
* `git clone https://github.com/mozilla/treeherder; cd treeherder`
* `vagrant up`
* `vagrant ssh`
  * `./bin/run_gunicorn`
* Outside of vm, open `http://local.treeherder.mozilla.org` and login to create an account
* `vagrant ssh`
  * `./manage.py create_credentials <username> <email> "description"`, the email has to match your logged in user. Remember to log-in through the Web UI once before you run this.
  * Setup your Treeherder client ID and secret as environment variables `TREEHERDER_CLIENT_ID` and `TREEHERDER_CLIENT_SECRET`

# Troubleshooting

 If you saw this error message:

```
venv/bin/activate: line 8: _OLD_VIRTUAL_PATH: unbound variable
```

That means your `virtualenv` is too old, try run `pip install -U virtualenv` to upgrade (If you installed ubuntu's `python-virtualenv` package, uninstall it first then install it through `pip`)
