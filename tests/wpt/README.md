This folder contains the web platform tests and the
code required to integrate them with Servo. 
To learn how to write tests, go [here](http://web-platform-tests.org/writing-tests/index.html).

Contents
========

In particular, this folder contains:

* `config.ini`: some configuration for the web-platform-tests.
* `include.ini`: the subset of web-platform-tests we currently run.
* `run.py`: run the web-platform-tests in Servo.
* `web-platform-tests`: copy of the web-platform-tests.
* `metadata`: expected failures for the web-platform-tests we run.
* `mozilla`: web-platform-tests that cannot be upstreamed.

Running the tests
=================

The simplest way to run the web-platform-tests in Servo is `./mach
test-wpt` in the root directory. This will run the subset of
JavaScript tests defined in `include.ini` and log the output to
stdout.

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

When avoiding `mach` for some reason, one can run `run.py`
directly. However, this requires that all the dependencies for
`wptrunner` are avaliable in the current python environment.

Running the tests manually
--------------------------

(See also [the relevant section of the upstream README][upstream-running].)

It can be useful to run a test without the interference of the test runner, for
example when using a debugger such as `gdb`.

To do this, first add the following to the system's hosts file:

    127.0.0.1   www.web-platform.test
    127.0.0.1   www1.web-platform.test
    127.0.0.1   www2.web-platform.test
    127.0.0.1   web-platform.test
    127.0.0.1   xn--n8j6ds53lwwkrqhv28a.web-platform.test
    127.0.0.1   xn--lve-6lad.web-platform.test

Then, navigate to `tests/wpt/web-platform-tests`. Next, create a directory,
e.g. `local-resources/`, to contain a local copy of the
`resources/testharnessreport.js` file. The version in the repository is
actually a Python format string that has substitution done on it by
`harness/wptrunner/environment.py` to configure test output. Then, place a
modified copy of the `testharnessreport.js` file in that directory, removing
the format string variable:

    mkdir local-resources
    cp resources/testharnessreport.js local-resources/
    $EDITOR local-resources/testharnessreport.js
    # Replace `output:%(output)d` with `output:1` or `output:0`.

Now create a configuration file at `config.json` for the web-platform-tests
server (configuration options you don't specify will be loaded from the
defaults at `config.default.json`) with the following contents:

    {"aliases": [
      {"url-path": "/resources/testharnessreport.js",
       "local-dir": "local-resources"
      }
     ]
    }

Finally, you can run `python serve` from `tests/wpt/web-platform-tests`.
Then navigate Servo to `http://web-platform.test:8000/path/to/test` or
`https://web-platform.test:8443/path/to/test`.

To prevent browser SSL warnings when running HTTPS tests locally,
you will need to run Servo with `--certificate-path resources/cert-wpt-only`.

[upstream-running]: https://github.com/w3c/web-platform-tests#running-the-tests

Running the tests in Firefox
----------------------------

When working with tests, you may want to compare Servo's result with Firefox.
You can supply `--product firefox` along with the path to a Firefox binary (as
well as few more odds and ends) to run tests in Firefox from your Servo
checkout:

    GECKO="$HOME/projects/mozilla/gecko"
    GECKO_BINS="$GECKO/obj-firefox-release-artifact/dist/Nightly.app/Contents/MacOS"
    ./mach test-wpt dom --product firefox --binary $GECKO_BINS/firefox --certutil-binary $GECKO_BINS/certutil --prefs-root $GECKO/testing/profiles

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

Writing new tests
=================

The simplest way to create a new test is to use the following command:

    ./mach create-wpt tests/wpt/path/to/new/test.html

This will create test.html in the appropriate directory using the WPT
template for JavaScript tests. Tests are written using [testharness.js](https://github.com/w3c/testharness.js/). Documentation can be found [here](http://testthewebforward.org/docs/testharness-library.html).
To create a new reference test instead, use the following:

    ./mach create-wpt --reftest tests/wpt/path/to/new/reftest.html --reference tests/wpt/path/to/reference.html

`reference.html` will be created if it does not exist, and `reftest.html`
will be created using the WPT reftest template. To know more about reftests, check [this](http://web-platform-tests.org/writing-tests/reftests.html).
These new tests can then be run in the following manner like any other WPT test:

    ./mach test-wpt tests/wpt/path/to/new/test.html
    ./mach test-wpt tests/wpt/path/to/new/reftest.html



Editing tests
=============

web-platform-tests may be edited in-place and the changes committed to
the servo tree. These changes will be upstreamed when the tests are
next synced.

Updating the upstream tests
===========================

In order to update the tests from upstream use the same mach update
commands. e.g. to update the web-platform-tests:

    ./mach update-wpt --sync
    ./mach test-wpt --log-raw=update.log
    ./mach update-wpt update.log

This should create two commits in your servo repository with the
updated tests and updated metadata.

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

    ./mach test-wpt --log-raw wpt.log

This file can then be fed into the
[reftest analyzer](https://hg.mozilla.org/mozilla-central/raw-file/tip/layout/tools/reftest/reftest-analyzer-structured.xhtml)
which will show all failing tests (not just those with unexpected results).
Note that this ingests logs in a different format to [original version of the
tool](https://hg.mozilla.org/mozilla-central/raw-file/tip/layout/tools/reftest/reftest-analyzer.xhtml)
written for gecko reftests.

The reftest analyzer allows pixel-level comparison of the test and reference
screenshots. Tests that both fail and have an unexpected result are marked
with a `!`.

Updating the WPT manifest
=========================

MANIFEST.json can be regenerated automatically with the mach command `update-manifest` e.g.

    ./mach update-manifest

This is equivalent to running

    ./mach test-wpt --manifest-update SKIP_TESTS
