Servo Page Load Time Test
==============

[Tracking Bug](https://github.com/servo/servo/issues/10452)

# Usage
## Build Servo
* Clone the servo repo
* Compile release build
* Run `git_log_to_json.sh` in the servo repo, save it as `revision.json`

## Prepare the local Perfherder
* Add `192.168.33.10    local.treeherder.mozilla.org` to `/etc/hosts`
* `git clone https://github.com/mozilla/treeherder; cd treeherder`
* `vagrant up`
* `vagrant ssh`
  * `./bin/run_gunicorn`
* Outside of vm, open `http://local.treeherder.mozilla.org` and login to create an account
* `vagrant ssh`
  * `./manage.py create_credentials slyu slyu@mozilla.com "description"`, the email has to match your logged in user
  * Copy the clinet secrent to your `runner.py`


## Prepare the test runner
* Clone this repo
* Download [tp5n.zip](http://people.mozilla.org/~jmaher/taloszips/zips/tp5n.zip), extract it to `page_load_test/`
* Put your `servo` binary, `revision.json` and `resources` folder in `servo/`
* Run `prepare_manifest.sh` to tranform the tp5n manifest to our format
* `virtualenv venv; source venv/bin/activate; pip install treeherder-client`
* Run `test_all.sh`

# How it works
* The testcase is from tp5, every testcase will run 20 times, and we take the median.
* Each testcase is a subtest on Perfherder, and their summary time is the geometric mean of all the subtests.

# Running in buildbot
* We have a `master.cfg` for you to run it in buildbot. You can use the `servo-linux1` vagrant VM setup from [servo/saltfs](https://github.com/servo/saltfs) to run it.
* You'll need to setup this repository manully in the VM, check the inline comments in `master.cfg` for detail.

# Python version
The `runner.py` needs to be run on Python3.4+ (for the `timeout` in `subprocess`). But the `submit_to_perfherder.py` still runs on Python2. We should migrate everthing to Python3 later.

# TODO
* Check which tp5 test runs forever
* Report to perfherder
