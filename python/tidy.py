# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

import contextlib
import os
import fnmatch
import itertools
import re
import StringIO
import sys
from licenseck import licenses

filetypes_to_check = [".rs", ".rc", ".cpp", ".c", ".h", ".py", ".toml", ".webidl"]
reftest_dir = "./tests/ref"
reftest_filetype = ".list"
python_dependencies = [
    "./python/dependencies/flake8-2.4.1-py2.py3-none-any.whl",
    "./python/dependencies/pep8-1.5.7-py2.py3-none-any.whl",
    "./python/dependencies/pyflakes-0.9.0-py2.py3-none-any.whl",
]

ignored_files = [
    # Upstream
    "./support/*",
    "./tests/wpt/*",
    "./python/mach/*",
    "./python/mozdebug/*",
    "./python/mozinfo/*",
    "./python/mozlog/*",
    "./python/toml/*",
    "./components/script/dom/bindings/codegen/parser/*",
    "./components/script/dom/bindings/codegen/ply/*",

    # Generated and upstream code combined with our own. Could use cleanup
    "./target/*",
    "./ports/gonk/src/native_window_glue.cpp",
    "./ports/cef/*",

    # MIT license
    "./components/util/deque/mod.rs",

    # Hidden files/directories
    "./.*",
]


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


EMACS_HEADER = "/* -*- Mode:"
VIM_HEADER = "/* vim:"


def check_license(file_name, contents):
    if file_name.endswith(".toml"):
        raise StopIteration
    while contents.startswith(EMACS_HEADER) or contents.startswith(VIM_HEADER):
        _, _, contents = contents.partition("\n")
    valid_license = any(contents.startswith(license) for license in licenses)
    acknowledged_bad_license = "xfail-license" in contents[:100]
    if not (valid_license or acknowledged_bad_license):
        yield (1, "incorrect license")


def check_length(idx, line):
    max_length = 120
    if len(line) >= max_length:
        yield (idx + 1, "Line is longer than %d characters" % max_length)


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


def check_by_line(file_name, contents):
    lines = contents.splitlines(True)
    for idx, line in enumerate(lines):
        errors = itertools.chain(
            check_length(idx, line),
            check_whitespace(idx, line),
            check_whatwg_url(idx, line),
        )
        for error in errors:
            yield error


def check_flake8(file_name, contents):
    from flake8.main import check_code

    if not file_name.endswith(".py"):
        raise StopIteration

    @contextlib.contextmanager
    def stdout_redirect(where):
        sys.stdout = where
        try:
            yield where
        finally:
            sys.stdout = sys.__stdout__

    ignore = {
        "W291",  # trailing whitespace; the standard tidy process will enforce no trailing whitespace
        "E501",  # 80 character line length; the standard tidy process will enforce line length
    }

    output = StringIO.StringIO()
    with stdout_redirect(output):
        check_code(contents, ignore=ignore)
    for error in output.getvalue().splitlines():
        _, line_num, _, message = error.split(":", 3)
        yield line_num, message.strip()


def check_toml(file_name, contents):
    if not file_name.endswith(".toml"):
        raise StopIteration
    contents = contents.splitlines(True)
    for idx, line in enumerate(contents):
        if line.find("*") != -1:
            yield (idx + 1, "found asterisk instead of minimum version number")


def check_rust(file_name, contents):
    if not file_name.endswith(".rs") or \
       file_name.endswith("properties.mako.rs") or \
       file_name.endswith("style/build.rs") or \
       file_name.endswith("unit/style/stylesheets.rs"):
        raise StopIteration
    contents = contents.splitlines(True)
    comment_depth = 0
    merged_lines = ''

    uses = []

    for idx, line in enumerate(contents):
        # simplify the analysis
        line = line.strip()

        # Simple heuristic to avoid common case of no comments.
        if '/' in line:
            comment_depth += line.count('/*')
            comment_depth -= line.count('*/')

        if line.endswith('\\'):
            merged_lines += line[:-1]
            continue
        if comment_depth:
            merged_lines += line
            continue
        if merged_lines:
            line = merged_lines + line
            merged_lines = ''

        # get rid of strings and chars because cases like regex expression, keep attributes
        if not line_is_attribute(line):
            line = re.sub('".*?"|\'.*?\'', '', line)

        # get rid of comments
        line = re.sub('//.*?$|/\*.*?$|^\*.*?$', '', line)

        # get rid of attributes that do not contain =
        line = re.sub('^#[A-Za-z0-9\(\)\[\]_]*?$', '', line)

        match = re.search(r",[A-Za-z0-9]", line)
        if match:
            yield (idx + 1, "missing space after ,")

        if line_is_attribute(line):
            pre_space_re = r"[A-Za-z0-9]="
            post_space_re = r"=[A-Za-z0-9\"]"
        else:
            # - not included because of scientific notation (1e-6)
            pre_space_re = r"[A-Za-z0-9][\+/\*%=]"
            # * not included because of dereferencing and casting
            # - not included because of unary negation
            post_space_re = r"[\+/\%=][A-Za-z0-9\"]"

        match = re.search(pre_space_re, line)
        if match and not is_associated_type(match, line, 1):
            yield (idx + 1, "missing space before %s" % match.group(0)[1])

        match = re.search(post_space_re, line)
        if match and not is_associated_type(match, line, 0):
            yield (idx + 1, "missing space after %s" % match.group(0)[0])

        match = re.search(r"\)->", line)
        if match:
            yield (idx + 1, "missing space before ->")

        match = re.search(r"->[A-Za-z]", line)
        if match:
            yield (idx + 1, "missing space after ->")

        # Avoid flagging ::crate::mod and `trait Foo : Bar`
        match = line.find(" :")
        if match != -1:
            if line[0:match].find('trait ') == -1 and line[match + 2] != ':':
                yield (idx + 1, "extra space before :")

        # Avoid flagging crate::mod
        match = re.search(r"[^:]:[A-Za-z]", line)
        if match:
            # Avoid flagging macros like $t1:expr
            if line[0:match.end()].rfind('$') == -1:
                yield (idx + 1, "missing space after :")

        match = re.search(r"[A-Za-z0-9\)]{", line)
        if match:
            yield (idx + 1, "missing space before {")

        # ignored cases like {} and }}
        match = re.search(r"[^\s{}]}", line)
        if match and not (line.startswith("use") or line.startswith("pub use")):
            yield (idx + 1, "missing space before }")

        # ignored cases like {} and {{
        match = re.search(r"{[^\s{}]", line)
        if match and not (line.startswith("use") or line.startswith("pub use")):
            yield (idx + 1, "missing space after {")

        # imports must be in the same line and alphabetically sorted
        if line.startswith("use "):
            use = line[4:]
            if not use.endswith(";"):
                yield (idx + 1, "use statement spans multiple lines")
            uses.append(use[:len(use) - 1])
        elif len(uses) > 0:
            sorted_uses = sorted(uses)
            for i in range(len(uses)):
                if sorted_uses[i] != uses[i]:
                    message = "use statement is not in alphabetical order"
                    expected = "\n\t\033[93mexpected: {}\033[0m".format(sorted_uses[i])
                    found = "\n\t\033[91mfound: {}\033[0m".format(uses[i])
                    yield (idx + 1 - len(uses) + i, message + expected + found)
            uses = []


# Avoid flagging <Item=Foo> constructs
def is_associated_type(match, line, index):
    open_angle = line[0:match.end()].rfind('<')
    close_angle = line[open_angle:].find('>') if open_angle != -1 else -1
    is_equals = match.group(0)[index] == '='
    generic_open = open_angle != -1 and open_angle < match.start()
    generic_close = close_angle != -1 and close_angle + open_angle >= match.end()
    return is_equals and generic_open and generic_close


def line_is_attribute(line):
    return re.search(r"#\[.*\]", line)


def check_webidl_spec(file_name, contents):
    # Sorted by this function (in pseudo-Rust). The idea is to group the same
    # organization together.
    # fn sort_standards(a: &Url, b: &Url) -> Ordering {
    #     let a_domain = a.domain().split(".");
    #     a_domain.pop();
    #     a_domain.reverse();
    #     let b_domain = b.domain().split(".");
    #     b_domain.pop();
    #     b_domain.reverse();
    #     for i in a_domain.into_iter().zip(b_domain.into_iter()) {
    #         match i.0.cmp(b.0) {
    #             Less => return Less,
    #             Greater => return Greater,
    #             _ => (),
    #         }
    #     }
    #     a_domain.path().cmp(b_domain.path())
    # }
    if not file_name.endswith(".webidl"):
        raise StopIteration
    standards = [
        "//www.khronos.org/registry/webgl/specs",
        "//developer.mozilla.org/en-US/docs/Web/API",
        "//dev.w3.org/2006/webapi",
        "//dev.w3.org/csswg",
        "//dev.w3.org/fxtf",
        "//dvcs.w3.org/hg",
        "//dom.spec.whatwg.org",
        "//domparsing.spec.whatwg.org",
        "//encoding.spec.whatwg.org",
        "//html.spec.whatwg.org",
        "//url.spec.whatwg.org",
        "//xhr.spec.whatwg.org",
        "//www.whatwg.org/html",
        "//www.whatwg.org/specs",
        "//w3c.github.io",
        # Not a URL
        "// This interface is entirely internal to Servo, and should not be" +
        " accessible to\n// web pages."
    ]
    for i in standards:
        if contents.find(i) != -1:
            raise StopIteration
    yield 0, "No specification link found."


def check_spec(file_name, contents):
    base_path = "components/script/dom/"
    if base_path not in file_name:
        raise StopIteration
    file_name = os.path.relpath(os.path.splitext(file_name)[0], base_path)
    patt = re.compile("^\s*\/\/.+")

    # Pattern representing a line with a macro
    macro_patt = re.compile("^\s*\S+!(.*)$")

    # Pattern representing a line with comment containing a spec link
    link_patt = re.compile("^\s*///? https://.+$")

    # Pattern representing a line with comment
    comment_patt = re.compile("^\s*///?.+$")

    pattern = "impl %sMethods for %s {" % (file_name, file_name)
    contents = contents.splitlines(True)
    brace_count = 0
    in_impl = False
    for idx, line in enumerate(contents):
        if "// check-tidy: no specs after this line" in line:
            break
        if not patt.match(line):
            if pattern.lower() in line.lower():
                in_impl = True
            if ("fn " in line or macro_patt.match(line)) and brace_count == 1:
                for up_idx in range(1, idx + 1):
                    up_line = contents[idx - up_idx]
                    if link_patt.match(up_line):
                        # Comment with spec link exists
                        break
                    if not comment_patt.match(up_line):
                        # No more comments exist above, yield warning
                        yield (idx + 1, "method declared in webidl is missing a comment with a specification link")
                        break
            if '{' in line and in_impl:
                brace_count += 1
            if '}' in line and in_impl:
                if brace_count == 1:
                    break
                brace_count -= 1


def collect_errors_for_files(files_to_check, checking_functions):
    base_path = "components/script/dom/"
    for file_name in files_to_check:
        with open(file_name, "r") as fp:
            contents = fp.read()
            for check in checking_functions:
                for error in check(file_name, contents):
                    # filename, line, message
                    yield (file_name, error[0], error[1])


def check_reftest_order(files_to_check):
    for file_name in files_to_check:
        with open(file_name, "r") as fp:
            split_lines = fp.read().splitlines()
            lines = filter(lambda l: len(l) > 0 and l[0] != '#', split_lines)
            for idx, line in enumerate(lines[:-1]):
                next_line = lines[idx + 1]
                current = get_reftest_names(line)
                next = get_reftest_names(next_line)
                if current is not None and next is not None and current > next:
                    yield (file_name, split_lines.index(next_line) + 1, "line not in alphabetical order")


def get_reftest_names(line):
    tokens = line.split()
    if len(tokens) == 3:
        return tokens[1] + tokens[2]
    if len(tokens) == 4:
        return tokens[2] + tokens[3]
    return None


def get_html_file_names_from_reftest_list(reftest_dir, file_name):
    for line in open(os.path.join(reftest_dir, file_name), "r"):
        for token in line.split():
            if fnmatch.fnmatch(token, '*.html'):
                yield os.path.join(reftest_dir, token)


def check_reftest_html_files_in_basic_list(reftest_dir):
    basic_list_files = set(get_html_file_names_from_reftest_list(reftest_dir, "basic" + reftest_filetype))

    for file_name in os.listdir(reftest_dir):
        file_path = os.path.join(reftest_dir, file_name)
        if fnmatch.fnmatch(file_path, '*.html') and file_path not in basic_list_files:
            yield (file_path, "", "not found in basic.list")


def scan():
    sys.path += python_dependencies

    all_files = (os.path.join(r, f) for r, _, files in os.walk(".") for f in files)
    files_to_check = filter(should_check, all_files)

    checking_functions = [check_license, check_by_line, check_flake8, check_toml,
                          check_rust, check_webidl_spec, check_spec]
    errors = collect_errors_for_files(files_to_check, checking_functions)

    reftest_files = (os.path.join(r, f) for r, _, files in os.walk(reftest_dir) for f in files)
    reftest_to_check = filter(should_check_reftest, reftest_files)
    r_errors = check_reftest_order(reftest_to_check)
    not_found_in_basic_list_errors = check_reftest_html_files_in_basic_list(reftest_dir)

    errors = list(itertools.chain(errors, r_errors, not_found_in_basic_list_errors))

    if errors:
        for error in errors:
            print "\033[94m{}\033[0m:\033[93m{}\033[0m: \033[91m{}\033[0m".format(*error)
        return 1
    else:
        print "\033[92mtidy reported no errors.\033[0m"
        return 0
