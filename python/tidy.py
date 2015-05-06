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
import re
from licenseck import licenses

directories_to_check = ["ports", "components", "tests"]
filetypes_to_check = [".rs", ".rc", ".cpp", ".c", ".h", ".py"]
reftest_directories = ["tests/ref"]
reftest_filetype = ".list"

ignored_files = [
    # Upstream
    "support/*",
    "tests/wpt/*",

    # Generated and upstream code combined with our own. Could use cleanup
    "components/script/dom/bindings/codegen/*",
    "components/style/properties/mod.rs",
    "components/servo/target/*",
    "ports/gonk/target/*",
    "ports/gonk/src/native_window_glue.cpp",
    "ports/cef/*",

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


def should_check_reftest(file_name):
    return file_name.endswith(reftest_filetype)


def check_license(contents):
    valid_license = any(contents.startswith(license) for license in licenses)
    acknowledged_bad_license = "xfail-license" in contents[:100]
    if not (valid_license or acknowledged_bad_license):
        yield (1, "incorrect license")


def check_length(idx, line):
    if len(line) >= 160:
        yield (idx + 1, "(much) overlong line")

def check_whatwg_url(idx, line):
    match = re.search(r"https://html\.spec\.whatwg\.org/multipage/[\w-]+\.html#([\w\:-]+)", line)
    if match is not None:
        preferred_link = "https://html.spec.whatwg.org/multipage/#{}".format(match.group(1))
        yield (idx + 1, "link to WHATWG may break in the future, use this format instead: {}".format(preferred_link))

def check_whitespace(idx, line):
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

def check_by_line(contents):
    lines = contents.splitlines(True)
    for idx, line in enumerate(lines):
        for error in itertools.chain(check_length(idx, line), check_whitespace(idx, line), check_whatwg_url(idx, line)):
            yield error


def collect_errors_for_files(files_to_check, checking_functions):
    for file_name in files_to_check:
        with open(file_name, "r") as fp:
            contents = fp.read()
            for check in checking_functions:
                for error in check(contents):
                    # filename, line, message
                    yield (file_name, error[0], error[1])


def check_reftest_order(files_to_check):
    for file_name in files_to_check:
        with open(file_name, "r") as fp:
            split_lines = fp.read().splitlines()
            lines = filter(lambda l: len(l) > 0 and l[0] != '#', split_lines)
            for idx, line in enumerate(lines[:-1]):
                next_line = lines[idx+1]
                current = get_reftest_names(line)
                next = get_reftest_names(next_line)
                if current is not None and next is not None and current > next:
                    yield (file_name, split_lines.index(next_line) + 1, "line not in alphabetical order")


def get_reftest_names(line):
    tokens = line.split()
    if (len(tokens) == 3):
        return tokens[1] + tokens[2]
    if (len(tokens) == 4):
        return tokens[2] + tokens[3]
    return None


def scan():
    all_files = collect_file_names(directories_to_check)
    files_to_check = filter(should_check, all_files)

    checking_functions = [check_license, check_by_line]
    errors = collect_errors_for_files(files_to_check, checking_functions)

    reftest_files = collect_file_names(reftest_directories)
    reftest_to_check = filter(should_check_reftest, reftest_files)
    r_errors = check_reftest_order(reftest_to_check)

    errors = list(itertools.chain(errors, r_errors))

    if errors:
        for error in errors:
            print("{}:{}: {}".format(*error))
        return 1
    else:
        print("tidy reported no errors.")
        return 0
