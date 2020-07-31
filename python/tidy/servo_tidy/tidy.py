# Copyright 2013 The Servo Project Developers. See the COPYRIGHT
# file at the top-level directory of this distribution.
#
# Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
# http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
# <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
# option. This file may not be copied, modified, or distributed
# except according to those terms.

from __future__ import print_function

import fnmatch
import glob
import imp
import itertools
import json
import os
import re
import subprocess
import sys

import colorama
import six
import toml
import voluptuous
import yaml
from .licenseck import OLD_MPL, MPL, APACHE, COPYRIGHT, licenses_toml, licenses_dep_toml
from six import iteritems
topdir = os.path.abspath(os.path.dirname(sys.argv[0]))
wpt = os.path.join(topdir, "tests", "wpt")


def wpt_path(*args):
    return os.path.join(wpt, *args)


CONFIG_FILE_PATH = os.path.join(".", "servo-tidy.toml")
WPT_MANIFEST_PATH = wpt_path("include.ini")
# regex source https://stackoverflow.com/questions/6883049/
URL_REGEX = re.compile(br'https?://(?:[-\w.]|(?:%[\da-fA-F]{2}))+')

# Import wptmanifest only when we do have wpt in tree, i.e. we're not
# inside a Firefox checkout.
if os.path.isfile(WPT_MANIFEST_PATH):
    sys.path.append(wpt_path("web-platform-tests", "tools", "wptrunner", "wptrunner"))
    from wptmanifest import parser, node

# Default configs
config = {
    "skip-check-length": False,
    "skip-check-licenses": False,
    "check-alphabetical-order": True,
    "check-ordered-json-keys": [],
    "lint-scripts": [],
    "blocked-packages": {},
    "ignore": {
        "files": [
            os.path.join(".", "."),   # ignore hidden files
        ],
        "directories": [
            os.path.join(".", "."),   # ignore hidden directories
        ],
        "packages": [],
    },
    "check_ext": {}
}

COMMENTS = [b"// ", b"# ", b" *", b"/* "]

# File patterns to include in the non-WPT tidy check.
FILE_PATTERNS_TO_CHECK = ["*.rs", "*.rc", "*.cpp", "*.c",
                          "*.h", "*.py", "*.sh",
                          "*.toml", "*.webidl", "*.json", "*.html",
                          "*.yml"]

# File patterns that are ignored for all tidy and lint checks.
FILE_PATTERNS_TO_IGNORE = ["*.#*", "*.pyc", "fake-ld.sh", "*.ogv", "*.webm"]

SPEC_BASE_PATH = "components/script/dom/"

WEBIDL_STANDARDS = [
    b"//www.khronos.org/registry/webgl/extensions",
    b"//www.khronos.org/registry/webgl/specs",
    b"//developer.mozilla.org/en-US/docs/Web/API",
    b"//dev.w3.org/2006/webapi",
    b"//dev.w3.org/csswg",
    b"//dev.w3.org/fxtf",
    b"//dvcs.w3.org/hg",
    b"//dom.spec.whatwg.org",
    b"//drafts.csswg.org",
    b"//drafts.css-houdini.org",
    b"//drafts.fxtf.org",
    b"//console.spec.whatwg.org",
    b"//encoding.spec.whatwg.org",
    b"//fetch.spec.whatwg.org",
    b"//html.spec.whatwg.org",
    b"//url.spec.whatwg.org",
    b"//xhr.spec.whatwg.org",
    b"//w3c.github.io",
    b"//heycam.github.io/webidl",
    b"//webbluetoothcg.github.io/web-bluetooth/",
    b"//svgwg.org/svg2-draft",
    b"//wicg.github.io",
    b"//webaudio.github.io",
    b"//immersive-web.github.io/",
    b"//github.com/immersive-web/webxr-test-api/",
    b"//github.com/immersive-web/webxr-hands-input/",
    b"//gpuweb.github.io",
    # Not a URL
    b"// This interface is entirely internal to Servo, and should not be"
    + b" accessible to\n// web pages."
]


def is_iter_empty(iterator):
    try:
        obj = next(iterator)
        return True, itertools.chain((obj,), iterator)
    except StopIteration:
        return False, iterator


def normilize_paths(paths):
    if isinstance(paths, six.string_types):
        return os.path.join(*paths.split('/'))
    else:
        return [os.path.join(*path.split('/')) for path in paths]


# A simple wrapper for iterators to show progress
# (Note that it's inefficient for giant iterators, since it iterates once to get the upper bound)
def progress_wrapper(iterator):
    list_of_stuff = list(iterator)
    total_files, progress = len(list_of_stuff), 0
    for idx, thing in enumerate(list_of_stuff):
        progress = int(float(idx + 1) / total_files * 100)
        sys.stdout.write('\r  Progress: %s%% (%d/%d)' % (progress, idx + 1, total_files))
        sys.stdout.flush()
        yield thing


class FileList(object):
    def __init__(self, directory, only_changed_files=False, exclude_dirs=[], progress=True):
        self.directory = directory
        self.excluded = exclude_dirs
        self.generator = self._filter_excluded() if exclude_dirs else self._default_walk()
        if only_changed_files:
            self.generator = self._git_changed_files()
        if progress:
            self.generator = progress_wrapper(self.generator)

    def _default_walk(self):
        for root, _, files in os.walk(self.directory):
            for f in files:
                yield os.path.join(root, f)

    def _git_changed_files(self):
        args = ["git", "log", "-n1", "--merges", "--format=%H"]
        last_merge = subprocess.check_output(args, universal_newlines=True).strip()
        if not last_merge:
            return

        args = ["git", "diff", "--name-only", last_merge, self.directory]
        file_list = normilize_paths(subprocess.check_output(args, universal_newlines=True).splitlines())

        for f in file_list:
            if not any(os.path.join('.', os.path.dirname(f)).startswith(path) for path in self.excluded):
                yield os.path.join('.', f)

    def _filter_excluded(self):
        for root, dirs, files in os.walk(self.directory, topdown=True):
            # modify 'dirs' in-place so that we don't do unnecessary traversals in excluded directories
            dirs[:] = [d for d in dirs if not any(os.path.join(root, d).startswith(name) for name in self.excluded)]
            for rel_path in files:
                yield os.path.join(root, rel_path)

    def __iter__(self):
        return self.generator

    def next(self):
        return next(self.generator)


def filter_file(file_name):
    if any(file_name.startswith(ignored_file) for ignored_file in config["ignore"]["files"]):
        return False
    base_name = os.path.basename(file_name)
    if any(fnmatch.fnmatch(base_name, pattern) for pattern in FILE_PATTERNS_TO_IGNORE):
        return False
    return True


def filter_files(start_dir, only_changed_files, progress):
    file_iter = FileList(start_dir, only_changed_files=only_changed_files,
                         exclude_dirs=config["ignore"]["directories"], progress=progress)
    # always yield Cargo.lock so that the correctness of transitive dependencies is checked
    yield "./Cargo.lock"

    for file_name in iter(file_iter):
        base_name = os.path.basename(file_name)
        if not any(fnmatch.fnmatch(base_name, pattern) for pattern in FILE_PATTERNS_TO_CHECK):
            continue
        if not filter_file(file_name):
            continue
        yield file_name


def uncomment(line):
    for c in COMMENTS:
        if line.startswith(c):
            if line.endswith(b"*/"):
                return line[len(c):(len(line) - 3)].strip()
            return line[len(c):].strip()


def is_apache_licensed(header):
    if APACHE in header:
        return any(c in header for c in COPYRIGHT)


def check_license(file_name, lines):
    if any(file_name.endswith(ext) for ext in (".yml", ".toml", ".lock", ".json", ".html")) or \
       config["skip-check-licenses"]:
        return

    if lines[0].startswith(b"#!") and lines[1].strip():
        yield (1, "missing blank line after shebang")

    blank_lines = 0
    max_blank_lines = 2 if lines[0].startswith(b"#!") else 1
    license_block = []

    for line in lines:
        line = line.rstrip(b'\n')
        if not line.strip():
            blank_lines += 1
            if blank_lines >= max_blank_lines:
                break
            continue
        line = uncomment(line)
        if line is not None:
            license_block.append(line)

    header = (b" ".join(license_block)).decode("utf-8")
    valid_license = OLD_MPL in header or MPL in header or is_apache_licensed(header)
    acknowledged_bad_license = "xfail-license" in header
    if not (valid_license or acknowledged_bad_license):
        yield (1, "incorrect license")


def check_modeline(file_name, lines):
    for idx, line in enumerate(lines[:5]):
        if re.search(b'^.*[ \t](vi:|vim:|ex:)[ \t]', line):
            yield (idx + 1, "vi modeline present")
        elif re.search(br'-\*-.*-\*-', line, re.IGNORECASE):
            yield (idx + 1, "emacs file variables present")


def check_length(file_name, idx, line):
    if any(file_name.endswith(ext) for ext in (".yml", ".lock", ".json", ".html", ".toml")) or \
       config["skip-check-length"]:
        return

    # Prefer shorter lines when shell scripting.
    max_length = 80 if file_name.endswith(".sh") else 120
    if len(line.rstrip(b'\n')) > max_length and not is_unsplittable(file_name, line):
        yield (idx + 1, "Line is longer than %d characters" % max_length)


def contains_url(line):
    return bool(URL_REGEX.search(line))


def is_unsplittable(file_name, line):
    return (
        contains_url(line)
        or file_name.endswith(".rs")
        and line.startswith(b"use ")
        and b"{" not in line
    )


def check_whatwg_specific_url(idx, line):
    match = re.search(br"https://html\.spec\.whatwg\.org/multipage/[\w-]+\.html#([\w\'\:-]+)", line)
    if match is not None:
        preferred_link = "https://html.spec.whatwg.org/multipage/#{}".format(match.group(1))
        yield (idx + 1, "link to WHATWG may break in the future, use this format instead: {}".format(preferred_link))


def check_whatwg_single_page_url(idx, line):
    match = re.search(br"https://html\.spec\.whatwg\.org/#([\w\'\:-]+)", line)
    if match is not None:
        preferred_link = "https://html.spec.whatwg.org/multipage/#{}".format(match.group(1))
        yield (idx + 1, "links to WHATWG single-page url, change to multi page: {}".format(preferred_link))


def check_whitespace(idx, line):
    if line.endswith(b"\n"):
        line = line[:-1]
    else:
        yield (idx + 1, "no newline at EOF")

    if line.endswith(b" "):
        yield (idx + 1, "trailing whitespace")

    if b"\t" in line:
        yield (idx + 1, "tab on line")

    if b"\r" in line:
        yield (idx + 1, "CR on line")


def check_by_line(file_name, lines):
    for idx, line in enumerate(lines):
        errors = itertools.chain(
            check_length(file_name, idx, line),
            check_whitespace(idx, line),
            check_whatwg_specific_url(idx, line),
            check_whatwg_single_page_url(idx, line),
        )

        for error in errors:
            yield error


def check_flake8(file_name, contents):
    if not file_name.endswith(".py"):
        return

    ignore = {
        "W291",  # trailing whitespace; the standard tidy process will enforce no trailing whitespace
        "W503",  # linebreak before binary operator; replaced by W504 - linebreak after binary operator
        "E501",  # 80 character line length; the standard tidy process will enforce line length
    }

    output = ""
    try:
        args = ["flake8", "--ignore=" + ",".join(ignore), file_name]
        subprocess.check_output(args, universal_newlines=True)
    except subprocess.CalledProcessError as e:
        output = e.output
    for error in output.splitlines():
        _, line_num, _, message = error.split(":", 3)
        yield line_num, message.strip()


def check_lock(file_name, contents):
    def find_reverse_dependencies(name, content):
        for package in itertools.chain([content.get("root", {})], content["package"]):
            for dependency in package.get("dependencies", []):
                parts = dependency.split()
                dependency = (parts[0], parts[1] if len(parts) > 1 else None, parts[2] if len(parts) > 2 else None)
                if dependency[0] == name:
                    yield package["name"], package["version"], dependency

    if not file_name.endswith(".lock"):
        return

    # Package names to be neglected (as named by cargo)
    exceptions = config["ignore"]["packages"]

    content = toml.loads(contents.decode("utf-8"))

    packages_by_name = {}
    for package in content.get("package", []):
        if "replace" in package:
            continue
        source = package.get("source", "")
        if source == r"registry+https://github.com/rust-lang/crates.io-index":
            source = "crates.io"
        packages_by_name.setdefault(package["name"], []).append((package["version"], source))

    for name in exceptions:
        if name not in packages_by_name:
            yield (1, "duplicates are allowed for `{}` but it is not a dependency".format(name))

    for (name, packages) in iteritems(packages_by_name):
        has_duplicates = len(packages) > 1
        duplicates_allowed = name in exceptions

        if has_duplicates == duplicates_allowed:
            continue

        if duplicates_allowed:
            message = 'duplicates for `{}` are allowed, but only single version found'.format(name)
        else:
            message = "duplicate versions for package `{}`".format(name)

        packages.sort()
        packages_dependencies = list(find_reverse_dependencies(name, content))
        for version, source in packages:
            short_source = source.split("#")[0].replace("git+", "")
            message += "\n\t\033[93mThe following packages depend on version {} from '{}':\033[0m" \
                       .format(version, short_source)
            for pname, package_version, dependency in packages_dependencies:
                if (not dependency[1] or version in dependency[1]) and \
                   (not dependency[2] or short_source in dependency[2]):
                    message += "\n\t\t" + pname + " " + package_version
        yield (1, message)

    # Check to see if we are transitively using any blocked packages
    blocked_packages = config["blocked-packages"]
    # Create map to keep track of visited exception packages
    visited_whitelisted_packages = {package: {} for package in blocked_packages.keys()}

    for package in content.get("package", []):
        package_name = package.get("name")
        package_version = package.get("version")
        for dependency in package.get("dependencies", []):
            dependency = dependency.split()
            dependency_name = dependency[0]
            whitelist = blocked_packages.get(dependency_name)
            if whitelist is not None:
                if package_name not in whitelist:
                    fmt = "Package {} {} depends on blocked package {}."
                    message = fmt.format(package_name, package_version, dependency_name)
                    yield (1, message)
                else:
                    visited_whitelisted_packages[dependency_name][package_name] = True

    # Check if all the exceptions to blocked packages actually depend on the blocked package
    for dependency_name, package_names in iteritems(blocked_packages):
        for package_name in package_names:
            if not visited_whitelisted_packages[dependency_name].get(package_name):
                fmt = "Package {} is not required to be an exception of blocked package {}."
                message = fmt.format(package_name, dependency_name)
                yield (1, message)


def check_toml(file_name, lines):
    if not file_name.endswith("Cargo.toml"):
        return
    ok_licensed = False
    for idx, line in enumerate(map(lambda line: line.decode("utf-8"), lines)):
        if idx == 0 and "[workspace]" in line:
            return
        line_without_comment, _, _ = line.partition("#")
        if line_without_comment.find("*") != -1:
            yield (idx + 1, "found asterisk instead of minimum version number")
        for license_line in licenses_toml:
            ok_licensed |= (license_line in line)
    if not ok_licensed:
        yield (0, ".toml file should contain a valid license.")


def check_shell(file_name, lines):
    if not file_name.endswith(".sh"):
        return

    shebang = "#!/usr/bin/env bash"
    required_options = ["set -o errexit", "set -o nounset", "set -o pipefail"]

    did_shebang_check = False

    if not lines:
        yield (0, 'script is an empty file')
        return

    if lines[0].rstrip() != shebang.encode("utf-8"):
        yield (1, 'script does not have shebang "{}"'.format(shebang))

    for idx, line in enumerate(map(lambda line: line.decode("utf-8"), lines[1:])):
        stripped = line.rstrip()
        # Comments or blank lines are ignored. (Trailing whitespace is caught with a separate linter.)
        if line.startswith("#") or stripped == "":
            continue

        if not did_shebang_check:
            if stripped in required_options:
                required_options.remove(stripped)
            else:
                # The first non-comment, non-whitespace, non-option line is the first "real" line of the script.
                # The shebang, options, etc. must come before this.
                if required_options:
                    formatted = ['"{}"'.format(opt) for opt in required_options]
                    yield (idx + 1, "script is missing options {}".format(", ".join(formatted)))
                did_shebang_check = True

        if "`" in stripped:
            yield (idx + 1, "script should not use backticks for command substitution")

        if " [ " in stripped or stripped.startswith("[ "):
            yield (idx + 1, "script should use `[[` instead of `[` for conditional testing")

        for dollar in re.finditer(r'\$', stripped):
            next_idx = dollar.end()
            if next_idx < len(stripped):
                next_char = stripped[next_idx]
                if not (next_char == '{' or next_char == '('):
                    yield(idx + 1, "variable substitutions should use the full \"${VAR}\" form")


def rec_parse(current_path, root_node):
    dirs = []
    for item in root_node.children:
        if isinstance(item, node.DataNode):
            next_depth = os.path.join(current_path, item.data)
            dirs.append(next_depth)
            dirs += rec_parse(next_depth, item)
    return dirs


def check_manifest_dirs(config_file, print_text=True):
    if not os.path.exists(config_file):
        yield(config_file, 0, "%s manifest file is required but was not found" % config_file)
        return

    # Load configs from include.ini
    with open(config_file, "rb") as content:
        conf_file = content.read()
        lines = conf_file.splitlines(True)

    if print_text:
        print('\rChecking the wpt manifest file...')

    p = parser.parse(lines)
    paths = rec_parse(wpt_path("web-platform-tests"), p)
    for idx, path in enumerate(paths):
        if '_mozilla' in path or '_webgl' in path:
            continue
        if not os.path.isdir(path):
            yield(config_file, idx + 1, "Path in manifest was not found: {}".format(path))


def check_rust(file_name, lines):
    if not file_name.endswith(".rs") or \
       file_name.endswith(".mako.rs") or \
       file_name.endswith(os.path.join("style", "build.rs")) or \
       file_name.endswith(os.path.join("unit", "style", "stylesheets.rs")):
        return

    comment_depth = 0
    merged_lines = ''
    import_block = False
    whitespace = False

    is_lib_rs_file = file_name.endswith("lib.rs")

    prev_open_brace = False
    multi_line_string = False
    prev_crate = {}
    prev_mod = {}
    prev_feature_name = ""
    indent = 0

    check_alphabetical_order = config["check-alphabetical-order"]
    decl_message = "{} is not in alphabetical order"
    decl_expected = "\n\t\033[93mexpected: {}\033[0m"
    decl_found = "\n\t\033[91mfound: {}\033[0m"

    for idx, original_line in enumerate(map(lambda line: line.decode("utf-8"), lines)):
        # simplify the analysis
        line = original_line.strip()
        indent = len(original_line) - len(line)

        is_attribute = re.search(r"#\[.*\]", line)
        is_comment = re.search(r"^//|^/\*|^\*", line)

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

        if multi_line_string:
            line, count = re.subn(
                r'^(\\.|[^"\\])*?"', '', line, count=1)
            if count == 1:
                multi_line_string = False
            else:
                continue

        # Ignore attributes, comments, and imports
        # Keep track of whitespace to enable checking for a merged import block
        if import_block:
            if not (is_comment or is_attribute or line.startswith("use ")):
                whitespace = line == ""

                if not whitespace:
                    import_block = False

        # get rid of strings and chars because cases like regex expression, keep attributes
        if not is_attribute and not is_comment:
            line = re.sub(r'"(\\.|[^\\"])*?"', '""', line)
            line = re.sub(
                r"'(\\.|[^\\']|(\\x[0-9a-fA-F]{2})|(\\u{[0-9a-fA-F]{1,6}}))'",
                "''", line)
            # If, after parsing all single-line strings, we still have
            # an odd number of double quotes, this line starts a
            # multiline string
            if line.count('"') % 2 == 1:
                line = re.sub(r'"(\\.|[^\\"])*?$', '""', line)
                multi_line_string = True

        # get rid of comments
        line = re.sub(r'//.*?$|/\*.*?$|^\*.*?$', '//', line)

        # get rid of attributes that do not contain =
        line = re.sub(r'^#[A-Za-z0-9\(\)\[\]_]*?$', '#[]', line)

        # flag this line if it matches one of the following regular expressions
        # tuple format: (pattern, format_message, filter_function(match, line))
        def no_filter(match, line):
            return True
        regex_rules = [
            # There should not be any extra pointer dereferencing
            (r": &Vec<", "use &[T] instead of &Vec<T>", no_filter),
            # No benefit over using &str
            (r": &String", "use &str instead of &String", no_filter),
            # There should be any use of banned types:
            # Cell<JSVal>, Cell<Dom<T>>, DomRefCell<Dom<T>>, DomRefCell<HEAP<T>>
            (r"(\s|:)+Cell<JSVal>", "Banned type Cell<JSVal> detected. Use MutDom<JSVal> instead", no_filter),
            (r"(\s|:)+Cell<Dom<.+>>", "Banned type Cell<Dom<T>> detected. Use MutDom<T> instead", no_filter),
            (r"DomRefCell<Dom<.+>>", "Banned type DomRefCell<Dom<T>> detected. Use MutDom<T> instead", no_filter),
            (r"DomRefCell<Heap<.+>>", "Banned type DomRefCell<Heap<T>> detected. Use MutDom<T> instead", no_filter),
            # No benefit to using &Root<T>
            (r": &Root<", "use &T instead of &Root<T>", no_filter),
            (r": &DomRoot<", "use &T instead of &DomRoot<T>", no_filter),
            (r"^&&", "operators should go at the end of the first line", no_filter),
            # -> () is unnecessary
            (r"-> \(\)", "encountered function signature with -> ()", no_filter),
        ]

        for pattern, message, filter_func in regex_rules:
            for match in re.finditer(pattern, line):
                if filter_func(match, line):
                    yield (idx + 1, message.format(*match.groups(), **match.groupdict()))

        if prev_open_brace and not line:
            yield (idx + 1, "found an empty line following a {")
        prev_open_brace = line.endswith("{")

        # check alphabetical order of extern crates
        if line.startswith("extern crate "):
            # strip "extern crate " from the begin and ";" from the end
            crate_name = line[13:-1]
            if indent not in prev_crate:
                prev_crate[indent] = ""
            if prev_crate[indent] > crate_name and check_alphabetical_order:
                yield(idx + 1, decl_message.format("extern crate declaration")
                      + decl_expected.format(prev_crate[indent])
                      + decl_found.format(crate_name))
            prev_crate[indent] = crate_name

        if line == "}":
            for i in [i for i in prev_crate.keys() if i > indent]:
                del prev_crate[i]

        # check alphabetical order of feature attributes in lib.rs files
        if is_lib_rs_file:
            match = re.search(r"#!\[feature\((.*)\)\]", line)

            if match:
                features = list(map(lambda w: w.strip(), match.group(1).split(',')))
                sorted_features = sorted(features)
                if sorted_features != features and check_alphabetical_order:
                    yield(idx + 1, decl_message.format("feature attribute")
                          + decl_expected.format(tuple(sorted_features))
                          + decl_found.format(tuple(features)))

                if prev_feature_name > sorted_features[0] and check_alphabetical_order:
                    yield(idx + 1, decl_message.format("feature attribute")
                          + decl_expected.format(prev_feature_name + " after " + sorted_features[0])
                          + decl_found.format(prev_feature_name + " before " + sorted_features[0]))

                prev_feature_name = sorted_features[0]
            else:
                # not a feature attribute line, so empty previous name
                prev_feature_name = ""

        # modules must be in the same line and alphabetically sorted
        if line.startswith("mod ") or line.startswith("pub mod "):
            # strip /(pub )?mod/ from the left and ";" from the right
            mod = line[4:-1] if line.startswith("mod ") else line[8:-1]

            if (idx - 1) < 0 or "#[macro_use]" not in lines[idx - 1].decode("utf-8"):
                match = line.find(" {")
                if indent not in prev_mod:
                    prev_mod[indent] = ""
                if match == -1 and not line.endswith(";"):
                    yield (idx + 1, "mod declaration spans multiple lines")
                if prev_mod[indent] and mod < prev_mod[indent] and check_alphabetical_order:
                    yield(idx + 1, decl_message.format("mod declaration")
                          + decl_expected.format(prev_mod[indent])
                          + decl_found.format(mod))
                prev_mod[indent] = mod
        else:
            # we now erase previous entries
            prev_mod = {}

        # derivable traits should be alphabetically ordered
        if is_attribute:
            # match the derivable traits filtering out macro expansions
            match = re.search(r"#\[derive\(([a-zA-Z, ]*)", line)
            if match:
                derives = list(map(lambda w: w.strip(), match.group(1).split(',')))
                # sort, compare and report
                sorted_derives = sorted(derives)
                if sorted_derives != derives and check_alphabetical_order:
                    yield(idx + 1, decl_message.format("derivable traits list")
                              + decl_expected.format(", ".join(sorted_derives))
                              + decl_found.format(", ".join(derives)))


# Avoid flagging <Item=Foo> constructs
def is_associated_type(match, line):
    if match.group(1) != '=':
        return False
    open_angle = line[0:match.end()].rfind('<')
    close_angle = line[open_angle:].find('>') if open_angle != -1 else -1
    generic_open = open_angle != -1 and open_angle < match.start()
    generic_close = close_angle != -1 and close_angle + open_angle >= match.end()
    return generic_open and generic_close


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
        return

    for i in WEBIDL_STANDARDS:
        if contents.find(i) != -1:
            return
    yield (0, "No specification link found.")


def duplicate_key_yaml_constructor(loader, node, deep=False):
    mapping = {}
    for key_node, value_node in node.value:
        key = loader.construct_object(key_node, deep=deep)
        if key in mapping:
            raise KeyError(key)
        value = loader.construct_object(value_node, deep=deep)
        mapping[key] = value
    return loader.construct_mapping(node, deep)


def lint_buildbot_steps_yaml(mapping):
    from voluptuous import Any, Extra, Required, Schema

    # Note: dictionary keys are optional by default in voluptuous
    env = Schema({Extra: str})
    commands = Schema([str])
    schema = Schema({
        'env': env,
        Extra: Any(
            commands,
            {
                'env': env,
                Required('commands'): commands,
            },
        ),
    })

    # Signals errors via exception throwing
    schema(mapping)


class SafeYamlLoader(yaml.SafeLoader):
    """Subclass of yaml.SafeLoader to avoid mutating the global SafeLoader."""
    pass


def check_yaml(file_name, contents):
    if not file_name.endswith("buildbot_steps.yml"):
        return

    # YAML specification doesn't explicitly disallow
    # duplicate keys, but they shouldn't be allowed in
    # buildbot_steps.yml as it could lead to confusion
    SafeYamlLoader.add_constructor(
        yaml.resolver.BaseResolver.DEFAULT_MAPPING_TAG,
        duplicate_key_yaml_constructor
    )

    try:
        contents = yaml.load(contents, Loader=SafeYamlLoader)
        lint_buildbot_steps_yaml(contents)
    except yaml.YAMLError as e:
        line = e.problem_mark.line + 1 if hasattr(e, 'problem_mark') else None
        yield (line, e)
    except KeyError as e:
        yield (None, "Duplicated Key ({})".format(e.args[0]))
    except voluptuous.MultipleInvalid as e:
        yield (None, str(e))


def check_for_possible_duplicate_json_keys(key_value_pairs):
    keys = [x[0] for x in key_value_pairs]
    seen_keys = set()
    for key in keys:
        if key in seen_keys:
            raise KeyError("Duplicated Key (%s)" % key)

        seen_keys.add(key)


def check_for_alphabetical_sorted_json_keys(key_value_pairs):
    for a, b in zip(key_value_pairs[:-1], key_value_pairs[1:]):
        if a[0] > b[0]:
            raise KeyError("Unordered key (found %s before %s)" % (a[0], b[0]))


def check_json_requirements(filename):
    def check_fn(key_value_pairs):
        check_for_possible_duplicate_json_keys(key_value_pairs)
        if filename in normilize_paths(config["check-ordered-json-keys"]):
            check_for_alphabetical_sorted_json_keys(key_value_pairs)
    return check_fn


def check_json(filename, contents):
    if not filename.endswith(".json"):
        return

    try:
        json.loads(contents, object_pairs_hook=check_json_requirements(filename))
    except ValueError as e:
        match = re.search(r"line (\d+) ", e.args[0])
        line_no = match and match.group(1)
        yield (line_no, e.args[0])
    except KeyError as e:
        yield (None, e.args[0])


def check_spec(file_name, lines):
    if SPEC_BASE_PATH not in file_name:
        return
    file_name = os.path.relpath(os.path.splitext(file_name)[0], SPEC_BASE_PATH)
    patt = re.compile(r"^\s*\/\/.+")

    # Pattern representing a line with a macro
    macro_patt = re.compile(r"^\s*\S+!(.*)$")

    # Pattern representing a line with comment containing a spec link
    link_patt = re.compile(r"^\s*///? (<https://.+>|https://.+)$")

    # Pattern representing a line with comment or attribute
    comment_patt = re.compile(r"^\s*(///?.+|#\[.+\])$")

    brace_count = 0
    in_impl = False
    pattern = "impl {}Methods for {} {{".format(file_name, file_name)

    for idx, line in enumerate(map(lambda line: line.decode("utf-8"), lines)):
        if "// check-tidy: no specs after this line" in line:
            break
        if not patt.match(line):
            if pattern.lower() in line.lower():
                in_impl = True
            if ("fn " in line or macro_patt.match(line)) and brace_count == 1:
                for up_idx in range(1, idx + 1):
                    up_line = lines[idx - up_idx].decode("utf-8")
                    if link_patt.match(up_line):
                        # Comment with spec link exists
                        break
                    if not comment_patt.match(up_line):
                        # No more comments exist above, yield warning
                        yield (idx + 1, "method declared in webidl is missing a comment with a specification link")
                        break
            if in_impl:
                brace_count += line.count('{')
                brace_count -= line.count('}')
                if brace_count < 1:
                    break


def check_config_file(config_file, print_text=True, no_wpt=False):
    # Check if config file exists
    if not os.path.exists(config_file):
        print("%s config file is required but was not found" % config_file)
        sys.exit(1)

    # Load configs from servo-tidy.toml
    with open(config_file) as content:
        conf_file = content.read()
        lines = conf_file.splitlines(True)

    if print_text:
        print('\rChecking the config file...')

    config_content = toml.loads(conf_file)
    exclude = config_content.get("ignore", {})

    # Check for invalid listed ignored directories
    exclude_dirs = [d for p in exclude.get("directories", []) for d in (glob.glob(p) or [p])]
    skip_dirs = ["./target", "./tests"]
    invalid_dirs = [d for d in exclude_dirs if not os.path.isdir(d) and not any(s in d for s in skip_dirs)]

    # Check for invalid listed ignored files
    invalid_files = [f for f in exclude.get("files", []) if not os.path.exists(f)]

    # Do not check for the existense of ignored files under tests/wpts if --no-wpt is used
    if no_wpt:
        wpt_dir = './tests/wpt/'
        invalid_files = [f for f in invalid_files if not os.path.commonprefix([wpt_dir, f]) == wpt_dir]

    current_table = ""
    for idx, line in enumerate(lines):
        # Ignore comment lines
        if line.strip().startswith("#"):
            continue

        # Check for invalid tables
        if re.match(r"\[(.*?)\]", line.strip()):
            table_name = re.findall(r"\[(.*?)\]", line)[0].strip()
            if table_name not in ("configs", "blocked-packages", "ignore", "check_ext"):
                yield config_file, idx + 1, "invalid config table [%s]" % table_name
            current_table = table_name
            continue

        # Print invalid listed ignored directories
        if current_table == "ignore" and invalid_dirs:
            for d in invalid_dirs:
                if line.strip().strip('\'",') == d:
                    yield config_file, idx + 1, "ignored directory '%s' doesn't exist" % d
                    invalid_dirs.remove(d)
                    break

        # Print invalid listed ignored files
        if current_table == "ignore" and invalid_files:
            for f in invalid_files:
                if line.strip().strip('\'",') == f:
                    yield config_file, idx + 1, "ignored file '%s' doesn't exist" % f
                    invalid_files.remove(f)
                    break

        # Skip if there is no equal sign in line, assuming it's not a key
        if "=" not in line:
            continue

        key = line.split("=")[0].strip()

        # Check for invalid keys inside [configs] and [ignore] table
        if (current_table == "configs" and key not in config
                or current_table == "ignore" and key not in config["ignore"]
                # Any key outside of tables
                or current_table == ""):
            yield config_file, idx + 1, "invalid config key '%s'" % key

    # Parse config file
    parse_config(config_content)


def parse_config(config_file):
    exclude = config_file.get("ignore", {})
    # Add list of ignored directories to config
    ignored_directories = [d for p in exclude.get("directories", []) for d in (glob.glob(p) or [p])]
    config["ignore"]["directories"] += normilize_paths(ignored_directories)
    # Add list of ignored files to config
    config["ignore"]["files"] += normilize_paths(exclude.get("files", []))
    # Add list of ignored packages to config
    config["ignore"]["packages"] = exclude.get("packages", [])

    # Add dict of dir, list of expected ext to config
    dirs_to_check = config_file.get("check_ext", {})
    # Fix the paths (OS-dependent)
    for path, exts in dirs_to_check.items():
        config['check_ext'][normilize_paths(path)] = exts

    # Add list of blocked packages
    config["blocked-packages"] = config_file.get("blocked-packages", {})

    # Override default configs
    user_configs = config_file.get("configs", [])
    for pref in user_configs:
        if pref in config:
            config[pref] = user_configs[pref]


def check_directory_files(directories, print_text=True):
    if print_text:
        print('\rChecking directories for correct file extensions...')
    for directory, file_extensions in directories.items():
        files = sorted(os.listdir(directory))
        for filename in files:
            if not any(filename.endswith(ext) for ext in file_extensions):
                details = {
                    "name": os.path.basename(filename),
                    "ext": ", ".join(file_extensions),
                    "dir_name": directory
                }
                message = '''Unexpected extension found for {name}. \
We only expect files with {ext} extensions in {dir_name}'''.format(**details)
                yield (filename, 1, message)


def collect_errors_for_files(files_to_check, checking_functions, line_checking_functions, print_text=True):
    (has_element, files_to_check) = is_iter_empty(files_to_check)
    if not has_element:
        return
    if print_text:
        print('\rChecking files for tidiness...')

    for filename in files_to_check:
        if not os.path.exists(filename):
            continue
        with open(filename, "rb") as f:
            contents = f.read()
            if not contents.strip():
                yield filename, 0, "file is empty"
                continue
            for check in checking_functions:
                for error in check(filename, contents):
                    # the result will be: `(filename, line, message)`
                    yield (filename,) + error
            lines = contents.splitlines(True)
            for check in line_checking_functions:
                for error in check(filename, lines):
                    yield (filename,) + error


def get_dep_toml_files(only_changed_files=False):
    if not only_changed_files:
        print('\nRunning the dependency licensing lint...')
        for root, directories, filenames in os.walk(".cargo"):
            for filename in filenames:
                if filename == "Cargo.toml":
                    yield os.path.join(root, filename)


def check_dep_license_errors(filenames, progress=True):
    filenames = progress_wrapper(filenames) if progress else filenames
    for filename in filenames:
        with open(filename, "r") as f:
            ok_licensed = False
            lines = f.readlines()
            for idx, line in enumerate(lines):
                for license_line in licenses_dep_toml:
                    ok_licensed |= (license_line in line)
            if not ok_licensed:
                yield (filename, 0, "dependency should contain a valid license.")


class LintRunner(object):
    def __init__(self, lint_path=None, only_changed_files=True,
                 exclude_dirs=[], progress=True, stylo=False, no_wpt=False):
        self.only_changed_files = only_changed_files
        self.exclude_dirs = exclude_dirs
        self.progress = progress
        self.path = lint_path
        self.stylo = stylo
        self.no_wpt = no_wpt

    def check(self):
        if not os.path.exists(self.path):
            yield (self.path, 0, "file does not exist")
            return
        if not self.path.endswith('.py'):
            yield (self.path, 0, "lint should be a python script")
            return
        dir_name, filename = os.path.split(self.path)
        sys.path.append(dir_name)
        module = imp.load_source(filename[:-3], self.path)
        sys.path.remove(dir_name)
        if not hasattr(module, 'Lint'):
            yield (self.path, 1, "script should contain a class named 'Lint'")
            return

        if not issubclass(module.Lint, LintRunner):
            yield (self.path, 1, "class 'Lint' should inherit from 'LintRunner'")
            return

        lint = module.Lint(self.path, self.only_changed_files,
                           self.exclude_dirs, self.progress, stylo=self.stylo, no_wpt=self.no_wpt)
        for error in lint.run():
            if type(error) is not tuple or (type(error) is tuple and len(error) != 3):
                yield (self.path, 1, "errors should be a tuple of (path, line, reason)")
                return
            yield error

    def get_files(self, path, **kwargs):
        args = ['only_changed_files', 'exclude_dirs', 'progress']
        kwargs = {k: kwargs.get(k, getattr(self, k)) for k in args}
        return FileList(path, **kwargs)

    def run(self):
        yield (self.path, 0, "class 'Lint' should implement 'run' method")


def run_lint_scripts(only_changed_files=False, progress=True, stylo=False, no_wpt=False):
    runner = LintRunner(only_changed_files=only_changed_files, progress=progress, stylo=stylo, no_wpt=no_wpt)
    for path in config['lint-scripts']:
        runner.path = path
        for error in runner.check():
            yield error


def scan(only_changed_files=False, progress=True, stylo=False, no_wpt=False):
    # check config file for errors
    config_errors = check_config_file(CONFIG_FILE_PATH, no_wpt=no_wpt)
    # check ini directories exist
    if not no_wpt and os.path.isfile(WPT_MANIFEST_PATH):
        manifest_errors = check_manifest_dirs(WPT_MANIFEST_PATH)
    else:
        manifest_errors = ()
    # check directories contain expected files
    directory_errors = check_directory_files(config['check_ext'])
    # standard checks
    files_to_check = filter_files('.', only_changed_files and not stylo, progress)
    checking_functions = (check_flake8, check_lock, check_webidl_spec, check_json, check_yaml)
    line_checking_functions = (check_license, check_by_line, check_toml, check_shell,
                               check_rust, check_spec, check_modeline)
    file_errors = collect_errors_for_files(files_to_check, checking_functions, line_checking_functions)
    # check dependecy licenses
    dep_license_errors = check_dep_license_errors(get_dep_toml_files(only_changed_files), progress)
    # other lint checks
    lint_errors = run_lint_scripts(only_changed_files, progress, stylo=stylo, no_wpt=no_wpt)
    # chain all the iterators
    errors = itertools.chain(config_errors, manifest_errors, directory_errors, lint_errors,
                             file_errors, dep_license_errors)

    error = None
    colorama.init()
    for error in errors:
        print("\r\033[94m{}\033[0m:\033[93m{}\033[0m: \033[91m{}\033[0m".format(*error))

    print()
    if error is None:
        print("\033[92mtidy reported no errors.\033[0m")

    return int(error is not None)
