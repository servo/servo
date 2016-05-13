Servo Page Load Time Test
==============

[Tracking Bug](https://github.com/servo/servo/issues/10452)

# Usage
## Build Servo
* Clone the servo repo
* Compile release build
* Run `git_log_to_json.sh` in the servo repo, save the output as `revision.json`

## Prepare the local Perfherder

If you want to test the data submission code in `submit_to_perfherder.py`, you can setup a local treeherder VM. If you don't need to test `submit_to_perfherder.py`, you can skip this step.

* Add `192.168.33.10    local.treeherder.mozilla.org` to `/etc/hosts`
* `git clone https://github.com/mozilla/treeherder; cd treeherder`
* `vagrant up`
* `vagrant ssh`
  * `./bin/run_gunicorn`
* Outside of vm, open `http://local.treeherder.mozilla.org` and login to create an account
* `vagrant ssh`
  * `./manage.py create_credentials slyu slyu@mozilla.com "description"`, the email has to match your logged in user. Remember to log-in through the Web UI once before you run this.
  * Open a file called `credential.json`. Copy the clinet secrent to your `credential.json`. You can use `credential.json.example` as a template.


## Prepare the test runner

* Clone this repo
* Download [tp5n.zip](http://people.mozilla.org/~jmaher/taloszips/zips/tp5n.zip), extract it to `page_load_test/`
* Put your `servo` binary, `revision.json` and `resources` folder in `servo/`
* Run `prepare_manifest.sh` to tranform the tp5n manifest to our format
* `virtualenv venv; source venv/bin/activate; pip install treeherder-client`
* Run `test_all.sh`
* Test results are submitted to https://treeherder.allizom.org/#/jobs?repo=servo

# How it works

* The testcase is from tp5, every testcase will run 20 times, and we take the median.
* Some of the tests will make Servo run forever, it's disabled right now. See https://github.com/servo/servo/issues/11087
* Each testcase is a subtest on Perfherder, and their summary time is the geometric mean of all the subtests.
* Notice that the test is different from the Talos TP5 test we run for Gecko. So you can NOT conclude that Servo is "faster" or "slower" than Gecko from this test.

# Running in buildbot

* We have a `master.cfg` for you to run it in buildbot. You can use the `servo-linux1` vagrant VM setup from [servo/saltfs](https://github.com/servo/saltfs) to run it.
* You'll need to setup this repository manully in the VM, check the inline comments in `master.cfg` for detail.

# Unit tests

The following tests can be run by `python -m pytest <filename>`:

* `test_runner.py`
* `test_submit_to_perfherder.py`

# Python version

The `runner.py` needs to be run on Python3.4+ (for the `timeout` in `subprocess`). But the `submit_to_perfherder.py` still runs on Python2. We should migrate everthing to Python3 later.

