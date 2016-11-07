Servo Page Load Time Test
==============

# Prerequisites

* Python3

# Basic Usage

## Prepare the test runner

* Clone this repo
* Download [tp5n.zip](http://people.mozilla.org/~jmaher/taloszips/zips/tp5n.zip), extract it to `page_load_test/tp5n`
* Run `prepare_manifest.sh` to transform the tp5n manifest to our format
* Install the Python3 `treeherder-client` package. For example, to install it in a virtualenv: `python3 -m virtualenv venv; source venv/bin/activate; pip install "treeherder-client>=3.0.0"`
* Setup your Treeherder client ID and secret as environment variables `TREEHERDER_CLIENT_ID` and `TREEHERDER_CLIENT_SECRET`

## Build Servo
* Clone the servo repo
* Compile release build
* Run `git_log_to_json.sh` in the servo repo, save the output as `revision.json`
* Put your `servo` binary, `revision.json` and `resources` folder in `etc/ci/performance/servo/`

## Run
* Activate the virutalenv: `source venv/bin/activate`
* Sync your system clock before running, the Perfherder API SSL check will fail if your system clock is not accurate. (e.g. `sudo nptdate tw.pool.ntp.org`)
* Run `test_all.sh [--servo|--gecko] [--submit]`
    - choose `servo` or `gecko` as the testing engine
    - enable `submit`, if you want to submit to perfherder
* Test results are submitted to https://treeherder.mozilla.org/#/jobs?repo=servo

# How it works

* The testcase is from tp5, every testcase will run 20 times, and we take the median.
* Some of the tests will make Servo run forever, it's disabled right now. See https://github.com/servo/servo/issues/11087
* Each testcase is a subtest on Perfherder, and their summary time is the geometric mean of all the subtests.
* Notice that the test is different from the Talos TP5 test we run for Gecko. So you can NOT conclude that Servo is "faster" or "slower" than Gecko from this test.

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

## For Gecko

* Install Firefox Nightly in your PATH
* Install [jpm](https://developer.mozilla.org/en-US/Add-ons/SDK/Tools/jpm#Installation)
* Run `jpm xpi` in the `firefox/addon` folder
* Install the generated `xpi` file to your Firefox Nightly

# Troubleshooting

 If you saw this error message:

```
venv/bin/activate: line 8: _OLD_VIRTUAL_PATH: unbound variable
```

That means your `virtualenv` is too old, try run `pip install -U virtualenv` to upgrade (If you installed ubuntu's `python-virtualenv` package, uninstall it first then install it through `pip`)
