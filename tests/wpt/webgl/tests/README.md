Welcome to the WebGL Conformance Test Suite
===========================================

Note: Before adding a new test or editing an existing test
[please read these guidelines](test-guidelines.md).

This is the WebGL conformance test suite. You can find a the current "live"
version at [https://www.khronos.org/registry/webgl/sdk/tests/webgl-conformance-tests.html](https://www.khronos.org/registry/webgl/sdk/tests/webgl-conformance-tests.html)

NOTE TO USERS: Unless you are a WebGL implementor, there is no need to submit
a conformance result using this process.  Should you discover bugs in your
browser's WebGL implementation, either via this test suite or otherwise,
please report them through your browser vendor's bug tracking system.

FOR WEBGL IMPLEMENTORS: Please follow the instructions below to create
a formal conformance submission.

1. Open webgl-conformance-tests.html in your target browser

2. Press the "run tests" button

3. At the end of the run, press "display text summary"

4. Verify that the User Agent and WebGL renderer strings identify your browser and target correctly.

5. Copy the contents of the text summary (starting with "WebGL Conformance Test Results") and send via email to
   webgl_conformance_submissions@khronos.org

Please see CONFORMANCE_RULES.txt in this directory for guidelines
about what constitutes a conformant WebGL implementation.

Usage Notes:
------------

There are various URL options you can pass in.

    run:         Set to 1 to start the tests automatically

                 Example: webgl-conformance-tests.html?run=1

    version:     Set to the version of the harness you wish to run. Tests
                 at this version or below will be run

                 Example: webgl-conformance-tests.html?version=1.3.2

    minVersion:  Set to the minimum version of each test to include. Only tests
                 at this version or above will be included.

                 Example: webgl-conformance-tests.html?minVersion=1.3.2

    fast:        Only run tests not marked with --slow

                 Example: webgl-conformance-tests.html?fast=true

    skip:        Comma separated list of regular expressions of which tests to skip.

                 Example: webgl-conformance-tests.html?skip=glsl,.*destruction\.html

    include:     Comma separated list of regular expressions of which tests to include.

                 Example: webgl-conformance-tests.html?include=glsl,.*destruction\.html

    frames:      The number of iframes to use to run tests in parallel.

                 Example: webgl-conformance-tests.html?frames=8

                 Note the tests are not required to run with anything other than frames = 1.

History
-------

The dates below are when work on the conformance suite version was started.

- 2011/02/24: Version 1.0.0
- 2012/02/23: Version 1.0.1
- 2012/03/20: Version 1.0.2
- 2013/02/14: Version 1.0.3
- 2013/10/11: Version 2.0.0
- 2014/11/14: Version 1.0.4
- 2016/11/21: Version 2.0.1