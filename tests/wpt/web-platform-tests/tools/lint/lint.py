from __future__ import print_function, unicode_literals

import abc
import argparse
import ast
import itertools
import json
import os
import re
import subprocess
import sys
import tempfile

from collections import defaultdict

from . import fnmatch
from .. import localpaths
from ..gitignore.gitignore import PathFilter
from ..wpt import testfiles

from manifest.sourcefile import SourceFile, js_meta_re, python_meta_re, space_chars, get_any_variants, get_default_any_variants
from six import binary_type, iteritems, itervalues
from six.moves import range
from six.moves.urllib.parse import urlsplit, urljoin

import logging

logger = None

def setup_logging(prefix=False):
    global logger
    if logger is None:
        logger = logging.getLogger(os.path.basename(os.path.splitext(__file__)[0]))
        handler = logging.StreamHandler(sys.stdout)
        # Only add a handler if the parent logger is missing a handler
        if logger.parent and len(logger.parent.handlers) == 0:
            handler = logging.StreamHandler(sys.stdout)
            logger.addHandler(handler)
    if prefix:
        format = logging.BASIC_FORMAT
    else:
        format = "%(message)s"
    formatter = logging.Formatter(format)
    for handler in logger.handlers:
        handler.setFormatter(formatter)
    logger.setLevel(logging.DEBUG)


setup_logging()


ERROR_MSG = """You must fix all errors; for details on how to fix them, see
http://web-platform-tests.org/writing-tests/lint-tool.html

However, instead of fixing a particular error, it's sometimes
OK to add a line to the lint.whitelist file in the root of the
web-platform-tests directory to make the lint tool ignore it.

For example, to make the lint tool ignore all '%s'
errors in the %s file,
you could add the following line to the lint.whitelist file.

%s: %s"""

def all_filesystem_paths(repo_root, subdir=None):
    path_filter = PathFilter(repo_root, extras=[".git/"])
    if subdir:
        expanded_path = subdir
    else:
        expanded_path = repo_root
    for dirpath, dirnames, filenames in os.walk(expanded_path):
        for filename in filenames:
            path = os.path.relpath(os.path.join(dirpath, filename), repo_root)
            if path_filter(path):
                yield path
        dirnames[:] = [item for item in dirnames if
                       path_filter(os.path.relpath(os.path.join(dirpath, item) + "/",
                                                   repo_root)+"/")]

def _all_files_equal(paths):
    """
    Checks all the paths are files that are byte-for-byte identical

    :param paths: the list of paths to compare
    :returns: True if they are all identical
    """
    paths = list(paths)
    if len(paths) < 2:
        return True

    first = paths.pop()
    size = os.path.getsize(first)
    if any(os.path.getsize(path) != size for path in paths):
        return False

    # Chunk this to avoid eating up memory and file descriptors
    bufsize = 4096*4  # 16KB, a "reasonable" number of disk sectors
    groupsize = 8  # Hypothesised to be large enough in the common case that everything fits in one group
    with open(first, "rb") as first_f:
        for start in range(0, len(paths), groupsize):
            path_group = paths[start:start+groupsize]
            first_f.seek(0)
            try:
                files = [open(x, "rb") for x in path_group]
                for _ in range(0, size, bufsize):
                    a = first_f.read(bufsize)
                    for f in files:
                        b = f.read(bufsize)
                        if a != b:
                            return False
            finally:
                for f in files:
                    f.close()

    return True


def check_path_length(repo_root, path):
    if len(path) + 1 > 150:
        return [("PATH LENGTH", "/%s longer than maximum path length (%d > 150)" % (path, len(path) + 1), path, None)]
    return []


def check_worker_collision(repo_root, path):
    endings = [(".any.html", ".any.js"),
               (".any.worker.html", ".any.js"),
               (".worker.html", ".worker.js")]
    for path_ending, generated in endings:
        if path.endswith(path_ending):
            return [("WORKER COLLISION",
                     "path ends with %s which collides with generated tests from %s files" % (path_ending, generated),
                     path,
                     None)]
    return []


def check_ahem_copy(repo_root, path):
    lpath = path.lower()
    if "ahem" in lpath and lpath.endswith(".ttf"):
        return [("AHEM COPY", "Don't add extra copies of Ahem, use /fonts/Ahem.ttf", path, None)]
    return []


def check_git_ignore(repo_root, paths):
    errors = []
    with tempfile.TemporaryFile('w+') as f:
        f.write('\n'.join(paths))
        f.seek(0)
        try:
            matches = subprocess.check_output(
                ["git", "check-ignore", "--verbose", "--no-index", "--stdin"], stdin=f)
            for match in matches.strip().split('\n'):
                match_filter, path = match.split()
                _, _, filter_string = match_filter.split(':')
                # If the matching filter reported by check-ignore is a special-case exception,
                # that's fine. Otherwise, it requires a new special-case exception.
                if filter_string[0] != '!':
                    errors += [("IGNORED PATH", "%s matches an ignore filter in .gitignore - "
                                "please add a .gitignore exception" % path, path, None)]
        except subprocess.CalledProcessError as e:
            # Nonzero return code means that no match exists.
            pass
    return errors


drafts_csswg_re = re.compile(r"https?\:\/\/drafts\.csswg\.org\/([^/?#]+)")
w3c_tr_re = re.compile(r"https?\:\/\/www\.w3c?\.org\/TR\/([^/?#]+)")
w3c_dev_re = re.compile(r"https?\:\/\/dev\.w3c?\.org\/[^/?#]+\/([^/?#]+)")


def check_css_globally_unique(repo_root, paths):
    """
    Checks that CSS filenames are sufficiently unique

    This groups files by path classifying them as "test", "reference", or
    "support".

    "test" files must have a unique name across files that share links to the
    same spec.

    "reference" and "support" files, on the other hand, must have globally
    unique names.

    :param repo_root: the repository root
    :param paths: list of all paths
    :returns: a list of errors found in ``paths``

    """
    test_files = defaultdict(set)
    ref_files = defaultdict(set)
    support_files = defaultdict(set)

    for path in paths:
        if os.name == "nt":
            path = path.replace("\\", "/")

        if not path.startswith("css/"):
            continue

        source_file = SourceFile(repo_root, path, "/")
        if source_file.name_is_non_test:
            # If we're name_is_non_test for a reason apart from support, ignore it.
            # We care about support because of the requirement all support files in css/ to be in
            # a support directory; see the start of check_parsed.
            offset = path.find("/support/")
            if offset == -1:
                continue

            parts = source_file.dir_path.split(os.path.sep)
            if (parts[0] in source_file.root_dir_non_test or
                any(item in source_file.dir_non_test - {"support"} for item in parts) or
                any(parts[:len(non_test_path)] == list(non_test_path) for non_test_path in source_file.dir_path_non_test)):
                continue

            name = path[offset+1:]
            support_files[name].add(path)
        elif source_file.name_is_reference:
            ref_files[source_file.name].add(path)
        else:
            test_files[source_file.name].add(path)

    errors = []

    for name, colliding in iteritems(test_files):
        if len(colliding) > 1:
            if not _all_files_equal([os.path.join(repo_root, x) for x in colliding]):
                # Only compute by_spec if there are prima-facie collisions because of cost
                by_spec = defaultdict(set)
                for path in colliding:
                    source_file = SourceFile(repo_root, path, "/")
                    for link in source_file.spec_links:
                        for r in (drafts_csswg_re, w3c_tr_re, w3c_dev_re):
                            m = r.match(link)
                            if m:
                                spec = m.group(1)
                                break
                        else:
                            continue
                        by_spec[spec].add(path)

                for spec, paths in iteritems(by_spec):
                    if not _all_files_equal([os.path.join(repo_root, x) for x in paths]):
                        for x in paths:
                            errors.append(("CSS-COLLIDING-TEST-NAME",
                                           "The filename %s in the %s testsuite is shared by: %s"
                                           % (name,
                                              spec,
                                              ", ".join(sorted(paths))),
                                           x,
                                           None))

    for error_name, d in [("CSS-COLLIDING-REF-NAME", ref_files),
                          ("CSS-COLLIDING-SUPPORT-NAME", support_files)]:
        for name, colliding in iteritems(d):
            if len(colliding) > 1:
                if not _all_files_equal([os.path.join(repo_root, x) for x in colliding]):
                    for x in colliding:
                        errors.append((error_name,
                                       "The filename %s is shared by: %s" % (name,
                                                                             ", ".join(sorted(colliding))),
                                       x,
                                       None))

    return errors


def parse_whitelist(f):
    """
    Parse the whitelist file given by `f`, and return the parsed structure.
    """

    data = defaultdict(lambda:defaultdict(set))
    ignored_files = set()

    for line in f:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = [item.strip() for item in line.split(":")]
        if len(parts) == 2:
            parts.append(None)
        else:
            parts[-1] = int(parts[-1])

        error_types, file_match, line_number = parts
        error_types = {item.strip() for item in error_types.split(",")}
        file_match = os.path.normcase(file_match)

        if "*" in error_types:
            ignored_files.add(file_match)
        else:
            for error_type in error_types:
                data[error_type][file_match].add(line_number)

    return data, ignored_files


def filter_whitelist_errors(data, errors):
    """
    Filter out those errors that are whitelisted in `data`.
    """

    if not errors:
        return []

    whitelisted = [False for item in range(len(errors))]

    for i, (error_type, msg, path, line) in enumerate(errors):
        normpath = os.path.normcase(path)
        # Allow whitelisting all lint errors except the IGNORED PATH lint,
        # which explains how to fix it correctly and shouldn't be ignored.
        if error_type in data and error_type != "IGNORED PATH":
            wl_files = data[error_type]
            for file_match, allowed_lines in iteritems(wl_files):
                if None in allowed_lines or line in allowed_lines:
                    if fnmatch.fnmatchcase(normpath, file_match):
                        whitelisted[i] = True

    return [item for i, item in enumerate(errors) if not whitelisted[i]]

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
    pattern = b"[ \t\f\v]$"
    error = "TRAILING WHITESPACE"
    description = "Whitespace at EOL"

class TabsRegexp(Regexp):
    pattern = b"^\t"
    error = "INDENT TABS"
    description = "Tabs used for indentation"

class CRRegexp(Regexp):
    pattern = b"\r$"
    error = "CR AT EOL"
    description = "CR character in line separator"

class SetTimeoutRegexp(Regexp):
    pattern = b"setTimeout\s*\("
    error = "SET TIMEOUT"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "setTimeout used; step_timeout should typically be used instead"

class W3CTestOrgRegexp(Regexp):
    pattern = b"w3c\-test\.org"
    error = "W3C-TEST.ORG"
    description = "External w3c-test.org domain used"

class WebPlatformTestRegexp(Regexp):
    pattern = b"web\-platform\.test"
    error = "WEB-PLATFORM.TEST"
    description = "Internal web-platform.test domain used"

class Webidl2Regexp(Regexp):
    pattern = b"webidl2\.js"
    error = "WEBIDL2.JS"
    description = "Legacy webidl2.js script used"

class ConsoleRegexp(Regexp):
    pattern = b"console\.[a-zA-Z]+\s*\("
    error = "CONSOLE"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "Console logging API used"

class GenerateTestsRegexp(Regexp):
    pattern = b"generate_tests\s*\("
    error = "GENERATE_TESTS"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "generate_tests used"

class PrintRegexp(Regexp):
    pattern = b"print(?:\s|\s*\()"
    error = "PRINT STATEMENT"
    file_extensions = [".py"]
    description = "Print function used"

class LayoutTestsRegexp(Regexp):
    pattern = b"eventSender|testRunner|window\.internals"
    error = "LAYOUTTESTS APIS"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "eventSender/testRunner/window.internals used; these are LayoutTests-specific APIs (WebKit/Blink)"

class SpecialPowersRegexp(Regexp):
    pattern = b"SpecialPowers"
    error = "SPECIALPOWERS API"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "SpecialPowers used; this is gecko-specific and not supported in wpt"


regexps = [item() for item in
           [TrailingWhitespaceRegexp,
            TabsRegexp,
            CRRegexp,
            SetTimeoutRegexp,
            W3CTestOrgRegexp,
            WebPlatformTestRegexp,
            Webidl2Regexp,
            ConsoleRegexp,
            GenerateTestsRegexp,
            PrintRegexp,
            LayoutTestsRegexp,
            SpecialPowersRegexp]]

def check_regexp_line(repo_root, path, f):
    errors = []

    applicable_regexps = [regexp for regexp in regexps if regexp.applies(path)]

    for i, line in enumerate(f):
        for regexp in applicable_regexps:
            if regexp.search(line):
                errors.append((regexp.error, regexp.description, path, i+1))

    return errors

def check_parsed(repo_root, path, f):
    source_file = SourceFile(repo_root, path, "/", contents=f.read())

    errors = []

    if path.startswith("css/"):
        if (source_file.type == "support" and
            not source_file.name_is_non_test and
            not source_file.name_is_reference):
            return [("SUPPORT-WRONG-DIR", "Support file not in support directory", path, None)]

        if (source_file.type != "support" and
            not source_file.name_is_reference and
            not source_file.spec_links):
            return [("MISSING-LINK", "Testcase file must have a link to a spec", path, None)]

    if source_file.name_is_non_test or source_file.name_is_manual:
        return []

    if source_file.markup_type is None:
        return []

    if source_file.root is None:
        return [("PARSE-FAILED", "Unable to parse file", path, None)]

    if source_file.type == "manual" and not source_file.name_is_manual:
        return [("CONTENT-MANUAL", "Manual test whose filename doesn't end in '-manual'", path, None)]

    if source_file.type == "visual" and not source_file.name_is_visual:
        return [("CONTENT-VISUAL", "Visual test whose filename doesn't end in '-visual'", path, None)]

    for reftest_node in source_file.reftest_nodes:
        href = reftest_node.attrib.get("href", "").strip(space_chars)
        parts = urlsplit(href)
        if (parts.scheme or parts.netloc) and parts != urlsplit("about:blank"):
            errors.append(("ABSOLUTE-URL-REF",
                     "Reference test with a reference file specified via an absolute URL: '%s'" % href, path, None))
            continue

        ref_url = urljoin(source_file.url, href)
        ref_parts = urlsplit(ref_url)

        if source_file.url == ref_url:
            errors.append(("SAME-FILE-REF",
                           "Reference test which points at itself as a reference",
                           path,
                           None))
            continue

        assert ref_parts.path != ""

        reference_file = os.path.join(repo_root, ref_parts.path[1:])
        reference_rel = reftest_node.attrib.get("rel", "")

        if not os.path.isfile(reference_file):
            errors.append(("NON-EXISTENT-REF",
                     "Reference test with a non-existent '%s' relationship reference: '%s'" % (reference_rel, href), path, None))

    if len(source_file.timeout_nodes) > 1:
        errors.append(("MULTIPLE-TIMEOUT", "More than one meta name='timeout'", path, None))

    for timeout_node in source_file.timeout_nodes:
        timeout_value = timeout_node.attrib.get("content", "").lower()
        if timeout_value != "long":
            errors.append(("INVALID-TIMEOUT", "Invalid timeout value %s" % timeout_value, path, None))

    if source_file.testharness_nodes:
        if len(source_file.testharness_nodes) > 1:
            errors.append(("MULTIPLE-TESTHARNESS",
                           "More than one <script src='/resources/testharness.js'>", path, None))

        testharnessreport_nodes = source_file.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testharnessreport.js']")
        if not testharnessreport_nodes:
            errors.append(("MISSING-TESTHARNESSREPORT",
                           "Missing <script src='/resources/testharnessreport.js'>", path, None))
        else:
            if len(testharnessreport_nodes) > 1:
                errors.append(("MULTIPLE-TESTHARNESSREPORT",
                               "More than one <script src='/resources/testharnessreport.js'>", path, None))

        testharnesscss_nodes = source_file.root.findall(".//{http://www.w3.org/1999/xhtml}link[@href='/resources/testharness.css']")
        if testharnesscss_nodes:
            errors.append(("PRESENT-TESTHARNESSCSS",
                           "Explicit link to testharness.css present", path, None))

        for element in source_file.variant_nodes:
            if "content" not in element.attrib:
                errors.append(("VARIANT-MISSING",
                               "<meta name=variant> missing 'content' attribute", path, None))
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
                                                    "timeout": len(source_file.timeout_nodes) > 0}.items()
                             if value]

        for elem in source_file.root.iter():
            if source_file.timeout_nodes and elem == source_file.timeout_nodes[0]:
                seen_elements["timeout"] = True
                if seen_elements["testharness"]:
                    errors.append(("LATE-TIMEOUT",
                                   "<meta name=timeout> seen after testharness.js script", path, None))

            elif elem == source_file.testharness_nodes[0]:
                seen_elements["testharness"] = True

            elif testharnessreport_nodes and elem == testharnessreport_nodes[0]:
                seen_elements["testharnessreport"] = True
                if not seen_elements["testharness"]:
                    errors.append(("EARLY-TESTHARNESSREPORT",
                                   "testharnessreport.js script seen before testharness.js script", path, None))

            if all(seen_elements[name] for name in required_elements):
                break

    if source_file.testdriver_nodes:
        if len(source_file.testdriver_nodes) > 1:
            errors.append(("MULTIPLE-TESTDRIVER",
                           "More than one <script src='/resources/testdriver.js'>", path, None))

        testdriver_vendor_nodes = source_file.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testdriver-vendor.js']")
        if not testdriver_vendor_nodes:
            errors.append(("MISSING-TESTDRIVER-VENDOR",
                           "Missing <script src='/resources/testdriver-vendor.js'>", path, None))
        else:
            if len(testdriver_vendor_nodes) > 1:
                errors.append(("MULTIPLE-TESTDRIVER-VENDOR",
                               "More than one <script src='/resources/testdriver-vendor.js'>", path, None))

    for element in source_file.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src]"):
        src = element.attrib["src"]
        for name in ["testharness", "testharnessreport", "testdriver", "testdriver-vendor"]:
            if "%s.js" % name == src or ("/%s.js" % name in src and src != "/resources/%s.js" % name):
                errors.append(("%s-PATH" % name.upper(), "%s.js script seen with incorrect path" % name, path, None))

    return errors

class ASTCheck(object):
    __metaclass__ = abc.ABCMeta
    error = None
    description = None

    @abc.abstractmethod
    def check(self, root):
        pass

class OpenModeCheck(ASTCheck):
    error = "OPEN-NO-MODE"
    description = "File opened without providing an explicit mode (note: binary files must be read with 'b' in the mode flags)"

    def check(self, root):
        errors = []
        for node in ast.walk(root):
            if isinstance(node, ast.Call):
                if hasattr(node.func, "id") and node.func.id in ("open", "file"):
                    if (len(node.args) < 2 and
                        all(item.arg != "mode" for item in node.keywords)):
                        errors.append(node.lineno)
        return errors

ast_checkers = [item() for item in [OpenModeCheck]]

def check_python_ast(repo_root, path, f):
    if not path.endswith(".py"):
        return []

    try:
        root = ast.parse(f.read())
    except SyntaxError as e:
        return [("PARSE-FAILED", "Unable to parse file", path, e.lineno)]

    errors = []
    for checker in ast_checkers:
        for lineno in checker.check(root):
            errors.append((checker.error, checker.description, path, lineno))
    return errors


broken_js_metadata = re.compile(b"//\s*META:")
broken_python_metadata = re.compile(b"#\s*META:")


def check_global_metadata(value):
    global_values = {item.strip() for item in value.split(b",") if item.strip()}

    included_variants = set.union(get_default_any_variants(),
                                  *(get_any_variants(v) for v in global_values if not v.startswith(b"!")))

    for global_value in global_values:
        if global_value.startswith(b"!"):
            excluded_value = global_value[1:]
            if not get_any_variants(excluded_value):
                yield ("UNKNOWN-GLOBAL-METADATA", "Unexpected value for global metadata")

            elif excluded_value in global_values:
                yield ("BROKEN-GLOBAL-METADATA", "Cannot specify both %s and %s" % (global_value, excluded_value))

            else:
                excluded_variants = get_any_variants(excluded_value)
                if not (excluded_variants & included_variants):
                    yield ("BROKEN-GLOBAL-METADATA", "Cannot exclude %s if it is not included" % (excluded_value,))

        else:
            if not get_any_variants(global_value):
                yield ("UNKNOWN-GLOBAL-METADATA", "Unexpected value for global metadata")


def check_script_metadata(repo_root, path, f):
    if path.endswith((".worker.js", ".any.js")):
        meta_re = js_meta_re
        broken_metadata = broken_js_metadata
    elif path.endswith(".py"):
        meta_re = python_meta_re
        broken_metadata = broken_python_metadata
    else:
        return []

    done = False
    errors = []
    for idx, line in enumerate(f):
        assert isinstance(line, binary_type), line

        m = meta_re.match(line)
        if m:
            key, value = m.groups()
            if key == b"global":
                errors.extend((kind, message, path, idx + 1) for (kind, message) in check_global_metadata(value))
            elif key == b"timeout":
                if value != b"long":
                    errors.append(("UNKNOWN-TIMEOUT-METADATA", "Unexpected value for timeout metadata", path, idx + 1))
            elif key == b"script":
                pass
            else:
                errors.append(("UNKNOWN-METADATA", "Unexpected kind of metadata", path, idx + 1))
        else:
            done = True

        if done:
            if meta_re.match(line):
                errors.append(("STRAY-METADATA", "Metadata comments should start the file", path, idx + 1))
            elif meta_re.search(line):
                errors.append(("INDENTED-METADATA", "Metadata comments should start the line", path, idx + 1))
            elif broken_metadata.search(line):
                errors.append(("BROKEN-METADATA", "Metadata comment is not formatted correctly", path, idx + 1))

    return errors


def check_path(repo_root, path):
    """
    Runs lints that check the file path.

    :param repo_root: the repository root
    :param path: the path of the file within the repository
    :returns: a list of errors found in ``path``
    """

    errors = []
    for path_fn in path_lints:
        errors.extend(path_fn(repo_root, path))
    return errors


def check_all_paths(repo_root, paths):
    """
    Runs lints that check all paths globally.

    :param repo_root: the repository root
    :param paths: a list of all the paths within the repository
    :returns: a list of errors found in ``f``
    """

    errors = []
    for paths_fn in all_paths_lints:
        errors.extend(paths_fn(repo_root, paths))
    return errors


def check_file_contents(repo_root, path, f):
    """
    Runs lints that check the file contents.

    :param repo_root: the repository root
    :param path: the path of the file within the repository
    :param f: a file-like object with the file contents
    :returns: a list of errors found in ``f``
    """

    errors = []
    for file_fn in file_lints:
        errors.extend(file_fn(repo_root, path, f))
        f.seek(0)
    return errors


def output_errors_text(errors):
    for error_type, description, path, line_number in errors:
        pos_string = path
        if line_number:
            pos_string += ":%s" % line_number
        logger.error("%s: %s (%s)" % (pos_string, description, error_type))


def output_errors_markdown(errors):
    if not errors:
        return
    heading = """Got lint errors:

| Error Type | Position | Message |
|------------|----------|---------|"""
    for line in heading.split("\n"):
        logger.error(line)
    for error_type, description, path, line_number in errors:
        pos_string = path
        if line_number:
            pos_string += ":%s" % line_number
        logger.error("%s | %s | %s |" % (error_type, pos_string, description))


def output_errors_json(errors):
    for error_type, error, path, line_number in errors:
        print(json.dumps({"path": path, "lineno": line_number,
                          "rule": error_type, "message": error}))


def output_error_count(error_count):
    if not error_count:
        return

    by_type = " ".join("%s: %d" % item for item in error_count.items())
    count = sum(error_count.values())
    logger.info("")
    if count == 1:
        logger.info("There was 1 error (%s)" % (by_type,))
    else:
        logger.info("There were %d errors (%s)" % (count, by_type))


def changed_files(wpt_root):
    revish = testfiles.get_revish(revish=None)
    changed, _ = testfiles.files_changed(revish, set(), include_uncommitted=True, include_new=True)
    return [os.path.relpath(item, wpt_root) for item in changed]


def lint_paths(kwargs, wpt_root):
    if kwargs.get("paths"):
        paths = []
        for path in kwargs.get("paths"):
            if os.path.isdir(path):
                path_dir = list(all_filesystem_paths(wpt_root, path))
                paths.extend(path_dir)
            elif os.path.isfile(path):
                paths.append(os.path.relpath(os.path.abspath(path), wpt_root))


    elif kwargs["all"]:
        paths = list(all_filesystem_paths(wpt_root))
    else:
        changed_paths = changed_files(wpt_root)
        force_all = False
        for path in changed_paths:
            path = path.replace(os.path.sep, "/")
            if path == "lint.whitelist" or path.startswith("tools/lint/"):
                force_all = True
                break
        paths = (list(changed_paths) if not force_all
                 else list(all_filesystem_paths(wpt_root)))

    return paths


def create_parser():
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="*",
                        help="List of paths to lint")
    parser.add_argument("--json", action="store_true",
                        help="Output machine-readable JSON format")
    parser.add_argument("--markdown", action="store_true",
                        help="Output markdown")
    parser.add_argument("--repo-root", help="The WPT directory. Use this"
                        "option if the lint script exists outside the repository")
    parser.add_argument("--all", action="store_true", help="If no paths are passed, try to lint the whole "
                        "working directory, not just files that changed")
    return parser


def main(**kwargs):
    if kwargs.get("json") and kwargs.get("markdown"):
        logger.critical("Cannot specify --json and --markdown")
        sys.exit(2)

    repo_root = kwargs.get('repo_root') or localpaths.repo_root
    output_format = {(True, False): "json",
                     (False, True): "markdown",
                     (False, False): "normal"}[(kwargs.get("json", False),
                                                kwargs.get("markdown", False))]

    if output_format == "markdown":
        setup_logging(True)

    paths = lint_paths(kwargs, repo_root)

    return lint(repo_root, paths, output_format)


def lint(repo_root, paths, output_format):
    error_count = defaultdict(int)
    last = None

    with open(os.path.join(repo_root, "lint.whitelist")) as f:
        whitelist, ignored_files = parse_whitelist(f)

    output_errors = {"json": output_errors_json,
                     "markdown": output_errors_markdown,
                     "normal": output_errors_text}[output_format]

    def process_errors(errors):
        """
        Filters and prints the errors, and updates the ``error_count`` object.

        :param errors: a list of error tuples (error type, message, path, line number)
        :returns: ``None`` if there were no errors, or
                  a tuple of the error type and the path otherwise
        """

        errors = filter_whitelist_errors(whitelist, errors)

        if not errors:
            return None

        output_errors(errors)
        for error_type, error, path, line in errors:
            error_count[error_type] += 1

        return (errors[-1][0], path)

    for path in paths[:]:
        abs_path = os.path.join(repo_root, path)
        if not os.path.exists(abs_path):
            paths.remove(path)
            continue

        if any(fnmatch.fnmatch(path, file_match) for file_match in ignored_files):
            paths.remove(path)
            continue

        errors = check_path(repo_root, path)
        last = process_errors(errors) or last

        if not os.path.isdir(abs_path):
            with open(abs_path, 'rb') as f:
                errors = check_file_contents(repo_root, path, f)
                last = process_errors(errors) or last

    errors = check_all_paths(repo_root, paths)
    last = process_errors(errors) or last

    if output_format in ("normal", "markdown"):
        output_error_count(error_count)
        if error_count:
            for line in (ERROR_MSG % (last[0], last[1], last[0], last[1])).split("\n"):
                logger.info(line)
    return sum(itervalues(error_count))

path_lints = [check_path_length, check_worker_collision, check_ahem_copy]
all_paths_lints = [check_css_globally_unique]
file_lints = [check_regexp_line, check_parsed, check_python_ast, check_script_metadata]

# Don't break users of the lint that don't have git installed.
try:
    subprocess.check_output(["git", "--version"])
    all_paths_lints += [check_git_ignore]
except subprocess.CalledProcessError:
    print('No git present; skipping .gitignore lint.')

if __name__ == "__main__":
    args = create_parser().parse_args()
    error_count = main(**vars(args))
    if error_count > 0:
        sys.exit(1)
