This directory contains the CSS build system.

It is recommended that it is run with `build.sh`, as
this ensures all dependencies are installed. Note that it is not
required to build the testsuites to run tests; you can just run tests
as with any other web-platform-tests tests (see ../../docs/).

The build system is formed of build.py in this directory, the
w3ctestlib package in w3ctestlib/, and the apiclient package in
apiclient/apiclient/. Note that apiclient exists as a separate
upstream project at https://hg.csswg.org/dev/apiclient/, and that
ideally any changes here should make it upstream.

Warning: The CSS build system is not tested in CI at all, so don't make any
changes without ensuring that it still works locally.
