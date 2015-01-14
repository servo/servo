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
import fnmatch
from licenseck import licenses

directories_to_check = ["src", "components"]
filetypes_to_check = [".rs", ".rc", ".cpp", ".c", ".h", ".py"]

ignored_files = [
    # Upstream
    "support/*",
    "tests/wpt/web-platform-tests/*",

    # Generated and upstream code combined with our own. Could use cleanup
    "components/script/dom/bindings/codegen/*",
    "components/style/properties/mod.rs",
    "components/servo/target/*",

    # MIT license
    "components/util/deque/mod.rs",
]


def collect_file_names(top_directories):
    for top_directory in top_directories:
        for dirname, dirs, files in os.walk(top_directory):
            for basename in files:
                yield os.path.join(dirname, basename)


def should_check(file_name):
    if ".#" in file_name:
        return False
    if os.path.splitext(file_name)[1] not in filetypes_to_check:
        return False
    for pattern in ignored_files:
        if fnmatch.fnmatch(file_name, pattern):
            return False
    return True


def check_license(contents):
    valid_license = any(contents.startswith(license) for license in licenses)
    acknowledged_bad_license = "xfail-license" in contents[:100]
    if not (valid_license or acknowledged_bad_license):
        yield (1, "incorrect license")


def check_whitespace(contents):
    lines = contents.splitlines(True)
    for idx, line in enumerate(lines):
        if line[-1] == "\n":
            line = line[:-1]
        else:
            yield (idx + 1, "no newline at EOF")

        if line.endswith(" "):
            yield (idx + 1, "trailing whitespace")

        if "\t" in line:
            yield (idx + 1, "tab on line")

        if "\r" in line:
            yield (idx + 1, "CR on line")


def collect_errors_for_files(files_to_check, checking_functions):
    for file_name in files_to_check:
        with open(file_name, "r") as fp:
            contents = fp.read()
            for check in checking_functions:
                for error in check(contents):
                    # filename, line, message
                    yield (file_name, error[0], error[1])


def scan():
    all_files = collect_file_names(directories_to_check)
    files_to_check = filter(should_check, all_files)

    checking_functions = [check_license, check_whitespace]
    errors = collect_errors_for_files(files_to_check, checking_functions)
    errors = list(errors)

    if errors:
        for error in errors:
            print("{}:{}: {}".format(*error))
        return 1
    else:
        return 0
