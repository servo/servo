This folder contains the web platform tests, CSS WG tests, and the
code required to integrate them with Servo.

Contents
========

In particular, this folder contains:

* `config.ini`: some configuration for the web-platform-tests.
* `include.ini`: the subset of web-platform-tests we currently run.
* `config_css.ini`: some configuration for the CSSWG tests.
* `include_css.ini`: the subset of the CSSWG tests we currently run.
* `run_wpt.py`: glue code to run the web-platform-tests in Servo.
* `run_css.py`: glue code to run the CSSWG tests in Servo.
* `run.py`: common code used by `run_wpt.py` and `run_css.py`.
* `web-platform-tests`: copy of the web-platform-tests.
* `metadata`: expected failures for the web-platform-tests we run.
* `css-tests`: copy of the built CSSWG tests.
* `metadata-css`: expected failures for the CSSWG tests we run.
* `mozilla`: web-platform-tests that cannot be upstreamed.

Running the tests
=================

The simplest way to run the web-platform-tests in Servo is `./mach
test-wpt` in the root directory. This will run the subset of
JavaScript tests defined in `include.ini` and log the output to
stdout.

Similarly the CSSWG tests can be run using `./mach test-css`.

A subset of tests may be run by providing positional arguments to the
mach command, either as filesystem paths or as test urls e.g.

    ./mach test-wpt tests/wpt/web-platform-tests/dom/historical.html

to run the dom/historical.html test, or

    ./mach test-wpt dom

to run all the DOM tests.

There are also a large number of command line options accepted by the
test harness; these are documented by running with `--help`.

Running all tests
------------------------------

Running all the WPT tests with debug mode results in a lot of timeout.
If one wants to run all the tests,
build with `mach build -r`
and
test with `mach test-wpt --release`

Running the tests without mach
------------------------------

When avoiding `mach` for some reason, one can run either `run_wpt.py`
ir `run_css.py` directly. However, this requires that all the
dependencies for `wptrunner` are avaliable in the current python
environment.

Running the tests manually
--------------------------

It can be useful to run a test without the interference of the test runner, for
example when using a debugger such as `gdb`. In that case, start the server by
first adding the following to the system's hosts file:

    127.0.0.1   www.web-platform.test
    127.0.0.1   www1.web-platform.test
    127.0.0.1   www2.web-platform.test
    127.0.0.1   web-platform.test
    127.0.0.1   xn--n8j6ds53lwwkrqhv28a.web-platform.test
    127.0.0.1   xn--lve-6lad.web-platform.test

and then running `python serve` from `tests/wpt/web-platform-tests`.
Then navigate Servo to `http://web-platform.test:8000/path/to/test`.

Updating test expectations
==========================

When fixing a bug that causes the result of a test to change, the expected
results for that test need to be changed. This can be done manually, by editing
the `.ini` file under the `metadata` folder that corresponds to the test. In
this case, remove the references to tests whose expectation is now `PASS`, and
remove `.ini` files that no longer contain any expectations.

When a larger number of changes is required, this process can be automated.
This first requires saving the raw, unformatted log from a test run, for
example by running `./mach test-wpt --log-raw /tmp/servo.log`. Once the
log is saved, run from the root directory:

    ./mach update-wpt /tmp/servo.log

For CSSWG tests a similar prcedure works, with `./mach test-css` and
`./mach update-css`.

Writing new tests
=================

The simplest way to create a new test is to use the following command:

    ./mach create-wpt tests/wpt/path/to/new/test.html

This will create test.html in the appropriate directory using the WPT
template for JavaScript tests. To create a new reference test instead,
use the following:

    ./mach create-wpt --reftest tests/wpt/path/to/new/reftest.html --reference tests/wpt/path/to/reference.html

`reference.html` will be created if it doesn't not exist, and `reftest.html`
will be created using the WPT reftest template. These new tests can then
be run in the following manner like any other WPT test:

    ./mach test-wpt tests/wpt/path/to/new/test.html
    ./mach test-wpt tests/wpt/path/to/new/reftest.html

Editing tests
=============

web-platform-tests may be edited in-place and the changes committed to
the servo tree. These changes will be upstreamed when the tests are
next synced.

For CSS tests this kind of in-place update is not possible because the
tests have a build step before they are pulled into the servo
repository. Therefore corrections must be submitted directly to the
source repository.

Updating the upstream tests
===========================

In order to update the tests from upstream use the same mach update
commands. e.g. to update the web-platform-tests:

    ./mach update-wpt --sync
    ./mach test-wpt --log-raw=update.log
    ./mach update-wpt update.log

This should create two commits in your servo repository with the
updated tests and updated metadata. The same process works for the
CSSWG tests, using the `update-css` and `test-css` mach commands.

Updating the test harness
=========================

The easiest way to update the test harness is using git:

    cd tests/wpt/harness
    git init .
    git remote add origin https://github.com/w3c/wptrunner
    git fetch origin
    git checkout -f origin/master
    cd ../../..

At this point you should commit the updated files in the *servo* git repository.

Servo-specific tests
====================

The `mozilla` directory contains tests that cannot be upstreamed for some
reason (e.g. because they depend on Servo-specific APIs), as well as some
legacy tests that should be upstreamed at some point. When run they are
mounted on the server under `/_mozilla/`.

Analyzing reftest results
=========================

Reftest results can be analyzed from a raw log file. To generate this run
with the `--log-raw` option e.g.

    ./mach test-css --log-raw css.log

This file can then be fed into the
[reftest analyzer](http://hoppipolla.co.uk/410/reftest-analyser-structured.xhtml)
which will show all failing tests (not just those with unexpected results).
Note that this ingests logs in a different format to original version of the
tool written for gecko reftests.

The reftest analyzer allows pixel-level comparison of the test and reference
screenshots. Tests that both fail and have an unexpected result are marked
with a `!`.
