# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

#!/usr/bin/env python

import os
import sys
from licenseck import check_license

err = 0


def report_error_name_no(name, no, s):
    global err
    print("%s:%d: %s" % (name, no, s))
    err = 1


def do_license_check(name, contents):
    if not check_license(name, contents):
        report_error_name_no(name, 1, "incorrect license")


def do_whitespace_check(name, contents):
    for idx, line in enumerate(contents):
        if line[-1] == "\n":
            line = line[:-1]
        else:
            report_error_name_no(name, idx + 1, "No newline at EOF")

        if line.endswith(' '):
            report_error_name_no(name, idx + 1, "trailing whitespace")

        if '\t' in line:
            report_error_name_no(name, idx + 1, "tab on line")

        if '\r' in line:
            report_error_name_no(name, idx + 1, "CR on line")


exceptions = [
    # Upstream
    "src/support",
    "src/platform",
    "src/compiler",
    "src/test/wpt/web-platform-tests",

    # Generated and upstream code combined with our own. Could use cleanup
    "src/components/script/dom/bindings/codegen",
    "src/components/style/properties/mod.rs",
]


def should_check(name):
    if ".#" in name:
        return False
    if not (name.endswith(".rs")
            or name.endswith(".rc")
            or name.endswith(".cpp")
            or name.endswith(".c")
            or name.endswith(".h")
            or name.endswith(".py")):
        return False
    for exception in exceptions:
        if exception in name:
            return False
    return True


file_names = []
for root, dirs, files in os.walk(sys.argv[1]):
    for myfile in files:
        file_name = root + "/" + myfile
        if should_check(file_name):
            file_names.append(file_name)

for path in file_names:
    with open(path, "r") as fp:
        lines = fp.readlines()
        do_license_check(path, "".join(lines))
        do_whitespace_check(path, lines)

sys.exit(err)
