This directory contains [Web IDL](https://heycam.github.io/webidl/) interface definitions for use in idlharness.js tests.

The `.idl` files are extracted from specs by [Reffy](https://github.com/tidoust/reffy) into [reffy-reports](https://github.com/tidoust/reffy-reports), and a [sync script](https://github.com/tidoust/reffy-reports/blob/master/wpt-sync/sync.js) sends [automatic PRs](https://github.com/web-platform-tests/wpt/pulls/autofoolip). The PRs require manual review but can be approved/merged by anyone with write access.

If some IDL in this directory is not up to date with the spec, first look for an [open PR](https://github.com/web-platform-tests/wpt/pulls/autofoolip) and if there is none [file an issue on reffy-reports](https://github.com/tidoust/reffy-reports/issues) and notify @foolip.
