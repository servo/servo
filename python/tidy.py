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
import itertools
from licenseck import licenses

directories_to_check = ["ports/gonk", "components"]
filetypes_to_check = [".rs", ".rc", ".cpp", ".c", ".h", ".py"]
reftest_filename = "tests/ref/basic.list"

ignored_files = [
    # Upstream
    "support/*",
    "tests/wpt/web-platform-tests/*",

    # Generated and upstream code combined with our own. Could use cleanup
    "components/script/dom/bindings/codegen/*",
    "components/style/properties/mod.rs",
    "components/servo/target/*",
    "ports/gonk/target/*",
    "ports/gonk/src/native_window_glue.cpp",

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


def check_length(contents):
    lines = contents.splitlines(True)
    for idx, line in enumerate(lines):
        if len(line) >= 160:
            yield (idx + 1, "(much) overlong line")


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

def check_reftest_order():
    with open(reftest_filename, "r") as fp:
        lines = contents = fp.read().splitlines()
        for idx, line in enumerate(lines[:-1]):
            next_line = lines[idx+1]

            # ignore empty lines
            if len(line) == 0 or len(next_line) == 0:
                continue

            # ignore commented out lines
            if line[0] == '#' or next_line[0] == '#':
                continue

            # ignore != and ==
            current = line[3:] if line[1] == '=' else line
            next = next_line[3:] if next_line[1] == '=' else next_line
            if current > next:
                yield (reftest_filename, idx + 1, "line not in alphabetical order")

def scan():
    all_files = collect_file_names(directories_to_check)
    files_to_check = filter(should_check, all_files)

    checking_functions = [check_license, check_length, check_whitespace]
    errors = collect_errors_for_files(files_to_check, checking_functions)
    r_errors = check_reftest_order()
    errors = list(itertools.chain(errors, r_errors))

    if errors:
        for error in errors:
            print("{}:{}: {}".format(*error))
        return 1
    else:
        print("tidy reported no errors.")
        return 0
