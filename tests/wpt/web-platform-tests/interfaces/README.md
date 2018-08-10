This directory contains [Web IDL](https://heycam.github.io/webidl/) interface definitions for use in idlharness.js tests.

The `.idl` files are extracted from specs by [Reffy](https://github.com/tidoust/reffy) into [reffy-reports](https://github.com/tidoust/reffy-reports), and then copied into this directory.

Automatically importing changes from reffy-reports is tracked by the [Auto-import IDL files](https://github.com/web-platform-tests/wpt/projects/1) project. Currently, it is only semi-automated, and not guaranteed to happen at any particular cadence. If you need to update an IDL file, please copy the file from [whatwg/idl/](https://github.com/tidoust/reffy-reports/tree/master/whatwg/idl) in reffy-reports.
