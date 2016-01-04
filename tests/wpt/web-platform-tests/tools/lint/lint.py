import os
import subprocess
import re
import sys
import fnmatch

from collections import defaultdict

from .. import localpaths
from manifest.sourcefile import SourceFile

here = os.path.abspath(os.path.split(__file__)[0])
repo_root = localpaths.repo_root

def git(command, *args):
    args = list(args)

    proc_kwargs = {"cwd": repo_root}

    command_line = ["git", command] + args

    try:
        return subprocess.check_output(command_line, **proc_kwargs)
    except subprocess.CalledProcessError:
        raise


def iter_files():
    for item in git("ls-tree", "-r", "--name-only", "HEAD").split("\n"):
        yield item


def check_path_length(path):
    if len(path) + 1 > 150:
        return [("PATH LENGTH", "/%s longer than maximum path length (%d > 150)" % (path, len(path) + 1), None)]
    return []

def set_type(error_type, errors):
    return [(error_type,) + error for error in errors]

def parse_whitelist_file(filename):
    data = defaultdict(lambda:defaultdict(set))

    with open(filename) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            parts = [item.strip() for item in line.split(":")]
            if len(parts) == 2:
                parts.append(None)
            else:
                parts[-1] = int(parts[-1])

            error_type, file_match, line_number = parts
            data[file_match][error_type].add(line_number)

    def inner(path, errors):
        whitelisted = [False for item in xrange(len(errors))]

        for file_match, whitelist_errors in data.iteritems():
            if fnmatch.fnmatch(path, file_match):
                for i, (error_type, msg, line) in enumerate(errors):
                    if "*" in whitelist_errors:
                        whitelisted[i] = True
                    elif error_type in whitelist_errors:
                        allowed_lines = whitelist_errors[error_type]
                        if None in allowed_lines or line in allowed_lines:
                            whitelisted[i] = True

        return [item for i, item in enumerate(errors) if not whitelisted[i]]
    return inner

_whitelist_fn = None
def whitelist_errors(path, errors):
    global _whitelist_fn

    if _whitelist_fn is None:
        _whitelist_fn = parse_whitelist_file(os.path.join(repo_root, "lint.whitelist"))
    return _whitelist_fn(path, errors)

class Regexp(object):
    pattern = None
    file_extensions = None
    error = None
    _re = None

    def __init__(self):
        self._re = re.compile(self.pattern)

    def applies(self, path):
        return (self.file_extensions is None or
                os.path.splitext(path)[1] in self.file_extensions)

    def search(self, line):
        return self._re.search(line)

class TrailingWhitespaceRegexp(Regexp):
    pattern = "[ \t\f\v]$"
    error = "TRAILING WHITESPACE"

class TabsRegexp(Regexp):
    pattern = "^\t"
    error = "INDENT TABS"

class CRRegexp(Regexp):
    pattern = "\r$"
    error = "CR AT EOL"

class W3CTestOrgRegexp(Regexp):
    pattern = "w3c\-test\.org"
    error = "W3C-TEST.ORG"

class Webidl2Regexp(Regexp):
    pattern = "webidl2\.js"
    error = "WEBIDL2.JS"

class ConsoleRegexp(Regexp):
    pattern = "console\.[a-zA-Z]+\s*\("
    error = "CONSOLE"
    file_extensions = [".html", ".htm", ".js", ".xht", ".html", ".svg"]

class PrintRegexp(Regexp):
    pattern = "print(?:\s|\s*\()"
    error = "PRINT STATEMENT"
    file_extensions = [".py"]

regexps = [item() for item in
           [TrailingWhitespaceRegexp,
            TabsRegexp,
            CRRegexp,
            W3CTestOrgRegexp,
            Webidl2Regexp,
            ConsoleRegexp,
            PrintRegexp]]

def check_regexp_line(path, f):
    errors = []

    applicable_regexps = [regexp for regexp in regexps if regexp.applies(path)]

    for i, line in enumerate(f):
        for regexp in applicable_regexps:
            if regexp.search(line):
                errors.append((regexp.error, "%s line %i" % (path, i+1), i+1))

    return errors

def check_parsed(path, f):
    source_file = SourceFile(repo_root, path, "/")

    errors = []

    if source_file.name_is_non_test or source_file.name_is_manual:
        return []

    if source_file.markup_type is None:
        return []

    if source_file.root is None:
        return [("PARSE-FAILED", "Unable to parse file %s" % path, None)]

    if len(source_file.timeout_nodes) > 1:
        errors.append(("MULTIPLE-TIMEOUT", "%s more than one meta name='timeout'" % path, None))

    for timeout_node in source_file.timeout_nodes:
        timeout_value = timeout_node.attrib.get("content", "").lower()
        if timeout_value != "long":
            errors.append(("INVALID-TIMEOUT", "%s invalid timeout value %s" % (path, timeout_value), None))

    if source_file.testharness_nodes:
        if len(source_file.testharness_nodes) > 1:
            errors.append(("MULTIPLE-TESTHARNESS",
                           "%s more than one <script src='/resources/testharness.js'>" % path, None))

        testharnessreport_nodes = source_file.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testharnessreport.js']")
        if not testharnessreport_nodes:
            errors.append(("MISSING-TESTHARNESSREPORT",
                           "%s missing <script src='/resources/testharnessreport.js'>" % path, None))
        else:
            if len(testharnessreport_nodes) > 1:
                errors.append(("MULTIPLE-TESTHARNESSREPORT",
                               "%s more than one <script src='/resources/testharnessreport.js'>" % path, None))

        for element in source_file.variant_nodes:
            if "content" not in element.attrib:
                errors.append(("VARIANT-MISSING",
                               "%s has <meta name=variant> missing 'content' attribute" % path, None))
            else:
                variant = element.attrib["content"]
                if variant != "" and variant[0] not in ("?", "#"):
                    errors.append(("MALFORMED-VARIANT",
                               "%s <meta name=variant> 'content' attribute must be the empty string or start with '?' or '#'" % path, None))

        seen_elements = {"timeout": False,
                         "testharness": False,
                         "testharnessreport": False}
        required_elements = [key for key, value in {"testharness": True,
                                                    "testharnessreport": len(testharnessreport_nodes) > 0,
                                                    "timeout": len(source_file.timeout_nodes) > 0}.iteritems()
                             if value]

        for elem in source_file.root.iter():
            if source_file.timeout_nodes and elem == source_file.timeout_nodes[0]:
                seen_elements["timeout"] = True
                if seen_elements["testharness"]:
                    errors.append(("LATE-TIMEOUT",
                                   "%s <meta name=timeout> seen after testharness.js script" % path, None))

            elif elem == source_file.testharness_nodes[0]:
                seen_elements["testharness"] = True

            elif testharnessreport_nodes and elem == testharnessreport_nodes[0]:
                seen_elements["testharnessreport"] = True
                if not seen_elements["testharness"]:
                    errors.append(("EARLY-TESTHARNESSREPORT",
                                   "%s testharnessreport.js script seen before testharness.js script" % path, None))

            if all(seen_elements[name] for name in required_elements):
                break

    return errors

def output_errors(errors):
    for error_type, error, line_number in errors:
        print "%s: %s" % (error_type, error)

def output_error_count(error_count):
    if not error_count:
        return

    by_type = " ".join("%s: %d" % item for item in error_count.iteritems())
    count = sum(error_count.values())
    if count == 1:
        print "There was 1 error (%s)" % (by_type,)
    else:
        print "There were %d errors (%s)" % (count, by_type)

def main():
    error_count = defaultdict(int)
    last = None

    def run_lint(path, fn, last, *args):
        errors = whitelist_errors(path, fn(path, *args))
        if errors:
            last = (errors[-1][0], path)

        output_errors(errors)
        for error_type, error, line in errors:
            error_count[error_type] += 1
        return last

    for path in iter_files():
        abs_path = os.path.join(repo_root, path)
        if not os.path.exists(path):
            continue
        for path_fn in path_lints:
            last = run_lint(path, path_fn, last)

        if not os.path.isdir(abs_path):
            with open(abs_path) as f:
                for file_fn in file_lints:
                    last = run_lint(path, file_fn, last, f)
                    f.seek(0)

    output_error_count(error_count)
    if error_count:
        print
        print "You must fix all errors; for details on how to fix them, see"
        print "https://github.com/w3c/web-platform-tests/blob/master/docs/lint-tool.md"
        print
        print "However, instead of fixing a particular error, it's sometimes"
        print "OK to add a line to the lint.whitelist file in the root of the"
        print "web-platform-tests directory to make the lint tool ignore it."
        print
        print "For example, to make the lint tool ignore all '%s'" % last[0]
        print "errors in the %s file," %  last[1]
        print "you could add the following line to the lint.whitelist file."
        print
        print "%s:%s" % (last[0], last[1])
    return sum(error_count.itervalues())

path_lints = [check_path_length]
file_lints = [check_regexp_line, check_parsed]

if __name__ == "__main__":
    error_count = main()
    if error_count > 0:
        sys.exit(1)
