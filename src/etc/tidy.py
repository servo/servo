# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

#!/usr/bin/env python

import fileinput, sys, os
from licenseck import *

err = 0

def report_error_name_no(name, no, s):
    global err
    print("%s:%d: %s" % (name, no, s))
    err=1

def report_err(s):
    report_error_name_no(fileinput.filename(), fileinput.filelineno(), s)

def report_warn(s):
    print("%s:%d: %s" % (fileinput.filename(),
                         fileinput.filelineno(),
                         s))

def do_license_check(name, contents):
    if not check_license(name, contents):
        report_error_name_no(name, 1, "incorrect license")

exceptions = [
    "src/support", # Upstream
    "src/platform", # Upstream
    "src/compiler", # Upstream
    "src/components/main/dom/bindings/codegen", # Generated and upstream code combined with our own. Could use cleanup
    "src/components/script/dom/bindings/codegen", # Generated and upstream code combined with our own. Could use cleanup
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

current_name = ""
current_contents = ""

for line in fileinput.input(file_names):
    if fileinput.isfirstline() and current_name != "":
        do_license_check(current_name, current_contents)

    if fileinput.isfirstline():
        current_name = fileinput.filename()
        current_contents = ""

    current_contents += line

if current_name != "":
    do_license_check(current_name, current_contents)

sys.exit(err)
