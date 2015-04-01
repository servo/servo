This directory contains various Python modules used to support servo
development.

= mach =

The command dispatch framework used to wrap the build system and test
harnesses.

= mozdebug =

mozbase module containing information about various debuggers.

This can be updated by copying the latest version from
https://hg.mozilla.org/mozilla-central/file/tip/testing/mozbase/mozdebug

= mozinfo =

Mozbase module for extracting information about the host hardware /
os.

This can be updated by copying the latest version from
hg.mozilla.org/mozilla-central/file/tip/testing/mozbase/mozinfo

= mozlog =

A mozbase logging module required for wptrunner output and command
line arguments.

This can be updated by copying the latest version from
hg.mozilla.org/mozilla-central/file/tip/testing/mozbase/mozlog

= servo =

servo-specific python code e.g. implementations of mach commands. This
is the canonical repository for this code.

== toml ==

Python module for reading toml files.

This can be updated from https://github.com/uiri/toml
