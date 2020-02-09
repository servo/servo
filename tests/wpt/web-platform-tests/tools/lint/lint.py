from __future__ import print_function, unicode_literals

import abc
import argparse
import ast
import json
import logging
import os
import re
import subprocess
import sys
import tempfile

from collections import defaultdict

from . import fnmatch
from . import rules
from .. import localpaths
from ..gitignore.gitignore import PathFilter
from ..wpt import testfiles
from ..manifest.vcs import walk

from ..manifest.sourcefile import SourceFile, js_meta_re, python_meta_re, space_chars, get_any_variants, get_default_any_variants
from six import binary_type, iteritems, itervalues, with_metaclass
from six.moves import range
from six.moves.urllib.parse import urlsplit, urljoin

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any
    from typing import Dict
    from typing import IO
    from typing import Iterable
    from typing import List
    from typing import Optional
    from typing import Sequence
    from typing import Set
    from typing import Text
    from typing import Tuple
    from typing import Type
    from typing import Union

    Whitelist = Dict[Text, Dict[Text, Set[Optional[int]]]]


logger = None  # type: Optional[logging.Logger]

def setup_logging(prefix=False):
    # type: (bool) -> None
    global logger
    if logger is None:
        logger = logging.getLogger(os.path.basename(os.path.splitext(__file__)[0]))
        handler = logging.StreamHandler(sys.stdout)  # type: logging.Handler
        # Only add a handler if the parent logger is missing a handler
        parent = logger.parent
        assert isinstance(parent, logging.Logger)
        if parent and len(parent.handlers) == 0:
            handler = logging.StreamHandler(sys.stdout)
            logger.addHandler(handler)
    if prefix:
        format = logging.BASIC_FORMAT
    else:
        format = str("%(message)s")
    formatter = logging.Formatter(format)
    for handler in logger.handlers:
        handler.setFormatter(formatter)
    logger.setLevel(logging.DEBUG)


setup_logging()


ERROR_MSG = """You must fix all errors; for details on how to fix them, see
https://web-platform-tests.org/writing-tests/lint-tool.html

However, instead of fixing a particular error, it's sometimes
OK to add a line to the lint.whitelist file in the root of the
web-platform-tests directory to make the lint tool ignore it.

For example, to make the lint tool ignore all '%s'
errors in the %s file,
you could add the following line to the lint.whitelist file.

%s: %s"""

def all_filesystem_paths(repo_root, subdir=None):
    # type: (str, Optional[str]) -> Iterable[str]
    path_filter = PathFilter(repo_root, extras=[str(".git/")])
    if subdir:
        expanded_path = subdir
    else:
        expanded_path = repo_root
    for dirpath, dirnames, filenames in path_filter(walk(expanded_path)):
        for filename, _ in filenames:
            path = os.path.join(dirpath, filename)
            if subdir:
                path = os.path.join(subdir, path)
            assert not os.path.isabs(path), path
            yield path


def _all_files_equal(paths):
    # type: (Iterable[str]) -> bool
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
    # type: (str, str) -> List[rules.Error]
    if len(path) + 1 > 150:
        return [rules.PathLength.error(path, (path, len(path) + 1))]
    return []


def check_file_type(repo_root, path):
    # type: (str, str) -> List[rules.Error]
    if os.path.islink(path):
        return [rules.FileType.error(path, (path, "symlink"))]
    return []


def check_worker_collision(repo_root, path):
    # type: (str, str) -> List[rules.Error]
    endings = [(".any.html", ".any.js"),
               (".any.worker.html", ".any.js"),
               (".worker.html", ".worker.js")]
    for path_ending, generated in endings:
        if path.endswith(path_ending):
            return [rules.WorkerCollision.error(path, (path_ending, generated))]
    return []


def check_gitignore_file(repo_root, path):
    # type: (str, str) -> List[rules.Error]
    if not path.endswith(".gitignore"):
        return []

    path_parts = path.split(os.path.sep)
    if len(path_parts) == 1:
        return []

    if path_parts[-1] != ".gitignore":
        return []

    if (path_parts[0] in ["tools", "docs"] or
        path_parts[:2] == ["resources", "webidl2"] or
        path_parts[:3] == ["css", "tools", "apiclient"]):
        return []

    return [rules.GitIgnoreFile.error(path)]


def check_ahem_copy(repo_root, path):
    # type: (str, str) -> List[rules.Error]
    lpath = path.lower()
    if "ahem" in lpath and lpath.endswith(".ttf"):
        return [rules.AhemCopy.error(path)]
    return []


def check_git_ignore(repo_root, paths):
    # type: (str, List[str]) -> List[rules.Error]
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
                    errors.append(rules.IgnoredPath.error(path, (path,)))
        except subprocess.CalledProcessError:
            # Nonzero return code means that no match exists.
            pass
    return errors


drafts_csswg_re = re.compile(r"https?\:\/\/drafts\.csswg\.org\/([^/?#]+)")
w3c_tr_re = re.compile(r"https?\:\/\/www\.w3c?\.org\/TR\/([^/?#]+)")
w3c_dev_re = re.compile(r"https?\:\/\/dev\.w3c?\.org\/[^/?#]+\/([^/?#]+)")


def check_css_globally_unique(repo_root, paths):
    # type: (str, List[str]) -> List[rules.Error]
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
    test_files = defaultdict(set)  # type: Dict[Union[bytes, Text], Set[str]]
    ref_files = defaultdict(set)  # type: Dict[Union[bytes, Text], Set[str]]
    support_files = defaultdict(set)  # type: Dict[Union[bytes, Text], Set[str]]

    for path in paths:
        if os.name == "nt":
            if isinstance(path, binary_type):
                path = path.replace(b"\\", b"/")
            else:
                path = path.replace(u"\\", u"/")

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

            support_name = path[offset+1:]
            support_files[support_name].add(path)
        elif source_file.name_is_reference:
            ref_files[source_file.name].add(path)
        else:
            test_name = source_file.name  # type: Union[bytes, Text]
            if isinstance(test_name, bytes):
                test_name = test_name.replace(b'-manual', b'')
            else:
                test_name = test_name.replace(u'-manual', u'')
            test_files[test_name].add(path)

    errors = []

    for name, colliding in iteritems(test_files):
        if len(colliding) > 1:
            if not _all_files_equal([os.path.join(repo_root, x) for x in colliding]):
                # Only compute by_spec if there are prima-facie collisions because of cost
                by_spec = defaultdict(set)  # type: Dict[Text, Set[str]]
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

                for spec, spec_paths in iteritems(by_spec):
                    if not _all_files_equal([os.path.join(repo_root, x) for x in spec_paths]):
                        for x in spec_paths:
                            context1 = (name, spec, ", ".join(sorted(spec_paths)))
                            errors.append(rules.CSSCollidingTestName.error(x,
                                                                           context1))

    for rule_class, d in [(rules.CSSCollidingRefName, ref_files),
                          (rules.CSSCollidingSupportName, support_files)]:
        for name, colliding in iteritems(d):
            if len(colliding) > 1:
                if not _all_files_equal([os.path.join(repo_root, x) for x in colliding]):
                    context2 = (name, ", ".join(sorted(colliding)))

                    for x in colliding:
                        errors.append(rule_class.error(x, context2))

    return errors


def parse_whitelist(f):
    # type: (IO[bytes]) -> Tuple[Whitelist, Set[Text]]
    """
    Parse the whitelist file given by `f`, and return the parsed structure.
    """

    data = defaultdict(lambda:defaultdict(set))  # type: Whitelist
    ignored_files = set()  # type: Set[Text]

    for line in f:
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = [item.strip() for item in line.split(":")]

        if len(parts) == 2:
            error_types_s, file_match = parts
            line_number = None  # type: Optional[int]
        else:
            error_types_s, file_match, line_number_s = parts
            line_number = int(line_number_s)

        error_types = {item.strip() for item in error_types_s.split(",")}
        file_match = os.path.normcase(file_match)

        if "*" in error_types:
            ignored_files.add(file_match)
        else:
            for error_type in error_types:
                data[error_type][file_match].add(line_number)

    return data, ignored_files


def filter_whitelist_errors(data, errors):
    # type: (Whitelist, Sequence[rules.Error]) -> List[rules.Error]
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


regexps = [item() for item in  # type: ignore
           [rules.TrailingWhitespaceRegexp,
            rules.TabsRegexp,
            rules.CRRegexp,
            rules.SetTimeoutRegexp,
            rules.W3CTestOrgRegexp,
            rules.WebPlatformTestRegexp,
            rules.Webidl2Regexp,
            rules.ConsoleRegexp,
            rules.GenerateTestsRegexp,
            rules.PrintRegexp,
            rules.LayoutTestsRegexp,
            rules.MissingDepsRegexp,
            rules.SpecialPowersRegexp]]


def check_regexp_line(repo_root, path, f):
    # type: (str, str, IO[bytes]) -> List[rules.Error]
    errors = []  # type: List[rules.Error]

    applicable_regexps = [regexp for regexp in regexps if regexp.applies(path)]

    for i, line in enumerate(f):
        for regexp in applicable_regexps:
            if regexp.search(line):
                errors.append((regexp.name, regexp.description, path, i+1))

    return errors


def check_parsed(repo_root, path, f):
    # type: (str, str, IO[bytes]) -> List[rules.Error]
    source_file = SourceFile(repo_root, path, "/", contents=f.read())

    errors = []  # type: List[rules.Error]

    if path.startswith("css/"):
        if (source_file.type == "support" and
            not source_file.name_is_non_test and
            not source_file.name_is_reference):
            return [rules.SupportWrongDir.error(path)]

        if (source_file.type != "support" and
            not source_file.name_is_reference and
            not source_file.spec_links):
            return [rules.MissingLink.error(path)]

    if source_file.name_is_non_test:
        return []

    if source_file.markup_type is None:
        return []

    if source_file.root is None:
        return [rules.ParseFailed.error(path)]

    if source_file.type == "manual" and not source_file.name_is_manual:
        errors.append(rules.ContentManual.error(path))

    if source_file.type == "visual" and not source_file.name_is_visual:
        errors.append(rules.ContentVisual.error(path))

    about_blank_parts = urlsplit("about:blank")
    for reftest_node in source_file.reftest_nodes:
        href = reftest_node.attrib.get("href", "").strip(space_chars)
        parts = urlsplit(href)

        if parts == about_blank_parts:
            continue

        if (parts.scheme or parts.netloc):
            errors.append(rules.AbsoluteUrlRef.error(path, (href,)))
            continue

        ref_url = urljoin(source_file.url, href)
        ref_parts = urlsplit(ref_url)

        if source_file.url == ref_url:
            errors.append(rules.SameFileRef.error(path))
            continue

        assert ref_parts.path != ""

        reference_file = os.path.join(repo_root, ref_parts.path[1:])
        reference_rel = reftest_node.attrib.get("rel", "")

        if not os.path.isfile(reference_file):
            errors.append(rules.NonexistentRef.error(path,
                                                     (reference_rel, href)))

    if len(source_file.timeout_nodes) > 1:
        errors.append(rules.MultipleTimeout.error(path))

    for timeout_node in source_file.timeout_nodes:
        timeout_value = timeout_node.attrib.get("content", "").lower()
        if timeout_value != "long":
            errors.append(rules.InvalidTimeout.error(path, (timeout_value,)))

    if source_file.testharness_nodes:
        test_type = source_file.manifest_items()[0]
        if test_type not in ("testharness", "manual"):
            errors.append(rules.TestharnessInOtherType.error(path, (test_type,)))
        if len(source_file.testharness_nodes) > 1:
            errors.append(rules.MultipleTestharness.error(path))

        testharnessreport_nodes = source_file.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testharnessreport.js']")
        if not testharnessreport_nodes:
            errors.append(rules.MissingTestharnessReport.error(path))
        else:
            if len(testharnessreport_nodes) > 1:
                errors.append(rules.MultipleTestharnessReport.error(path))

        for element in source_file.variant_nodes:
            if "content" not in element.attrib:
                errors.append(rules.VariantMissing.error(path))
            else:
                variant = element.attrib["content"]
                if variant != "" and variant[0] not in ("?", "#"):
                    errors.append(rules.MalformedVariant.error(path, (path,)))

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
                    errors.append(rules.LateTimeout.error(path))

            elif elem == source_file.testharness_nodes[0]:
                seen_elements["testharness"] = True

            elif testharnessreport_nodes and elem == testharnessreport_nodes[0]:
                seen_elements["testharnessreport"] = True
                if not seen_elements["testharness"]:
                    errors.append(rules.EarlyTestharnessReport.error(path))

            if all(seen_elements[name] for name in required_elements):
                break

    if source_file.testdriver_nodes:
        if len(source_file.testdriver_nodes) > 1:
            errors.append(rules.MultipleTestdriver.error(path))

        testdriver_vendor_nodes = source_file.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src='/resources/testdriver-vendor.js']")
        if not testdriver_vendor_nodes:
            errors.append(rules.MissingTestdriverVendor.error(path))
        else:
            if len(testdriver_vendor_nodes) > 1:
                errors.append(rules.MultipleTestdriverVendor.error(path))

    for element in source_file.root.findall(".//{http://www.w3.org/1999/xhtml}script[@src]"):
        src = element.attrib["src"]

        def incorrect_path(script, src):
            # type: (Text, Text) -> bool
            return (script == src or
                ("/%s" % script in src and src != "/resources/%s" % script))

        if incorrect_path("testharness.js", src):
            errors.append(rules.TestharnessPath.error(path))

        if incorrect_path("testharnessreport.js", src):
            errors.append(rules.TestharnessReportPath.error(path))

        if incorrect_path("testdriver.js", src):
            errors.append(rules.TestdriverPath.error(path))

        if incorrect_path("testdriver-vendor.js", src):
            errors.append(rules.TestdriverVendorPath.error(path))

    return errors

class ASTCheck(with_metaclass(abc.ABCMeta)):
    @abc.abstractproperty
    def rule(self):
        # type: () -> Type[rules.Rule]
        pass

    @abc.abstractmethod
    def check(self, root):
        # type: (ast.AST) -> List[int]
        pass

class OpenModeCheck(ASTCheck):
    rule = rules.OpenNoMode

    def check(self, root):
        # type: (ast.AST) -> List[int]
        errors = []
        for node in ast.walk(root):
            if isinstance(node, ast.Call):
                if hasattr(node.func, "id") and node.func.id in ("open", "file"):  # type: ignore
                    if (len(node.args) < 2 and
                        all(item.arg != "mode" for item in node.keywords)):
                        errors.append(node.lineno)
        return errors

ast_checkers = [item() for item in [OpenModeCheck]]

def check_python_ast(repo_root, path, f):
    # type: (str, str, IO[bytes]) -> List[rules.Error]
    if not path.endswith(".py"):
        return []

    try:
        root = ast.parse(f.read())
    except SyntaxError as e:
        return [rules.ParseFailed.error(path, line_no=e.lineno)]

    errors = []
    for checker in ast_checkers:
        for lineno in checker.check(root):
            errors.append(checker.rule.error(path, line_no=lineno))
    return errors


broken_js_metadata = re.compile(br"//\s*META:")
broken_python_metadata = re.compile(br"#\s*META:")


def check_global_metadata(value):
    # type: (str) -> Iterable[Tuple[Type[rules.Rule], Tuple[Any, ...]]]
    global_values = {item.strip() for item in value.split(b",") if item.strip()}

    included_variants = set.union(get_default_any_variants(),
                                  *(get_any_variants(v) for v in global_values if not v.startswith(b"!")))

    for global_value in global_values:
        if global_value.startswith(b"!"):
            excluded_value = global_value[1:]
            if not get_any_variants(excluded_value):
                yield (rules.UnknownGlobalMetadata, ())

            elif excluded_value in global_values:
                yield (rules.BrokenGlobalMetadata,
                       (("Cannot specify both %s and %s" % (global_value, excluded_value)),))

            else:
                excluded_variants = get_any_variants(excluded_value)
                if not (excluded_variants & included_variants):
                    yield (rules.BrokenGlobalMetadata,
                           (("Cannot exclude %s if it is not included" % (excluded_value,)),))

        else:
            if not get_any_variants(global_value):
                yield (rules.UnknownGlobalMetadata, ())


def check_script_metadata(repo_root, path, f):
    # type: (str, str, IO[bytes]) -> List[rules.Error]
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
                for rule_class, context in check_global_metadata(value):
                    errors.append(rule_class.error(path, context, idx + 1))
            elif key == b"timeout":
                if value != b"long":
                    errors.append(rules.UnknownTimeoutMetadata.error(path,
                                                                     line_no=idx + 1))
            elif key == b"title":
                pass
            elif key == b"script":
                pass
            elif key == b"variant":
                pass
            else:
                errors.append(rules.UnknownMetadata.error(path,
                                                          line_no=idx + 1))
        else:
            done = True

        if done:
            if meta_re.match(line):
                errors.append(rules.StrayMetadata.error(path, line_no=idx + 1))
            elif meta_re.search(line):
                errors.append(rules.IndentedMetadata.error(path,
                                                           line_no=idx + 1))
            elif broken_metadata.search(line):
                errors.append(rules.BrokenMetadata.error(path, line_no=idx + 1))

    return errors


ahem_font_re = re.compile(b"font.*:.*ahem", flags=re.IGNORECASE)
# Ahem can appear either in the global location or in the support
# directory for legacy Mozilla imports
ahem_stylesheet_re = re.compile(b"\/fonts\/ahem\.css|support\/ahem.css",
                                flags=re.IGNORECASE)


def check_ahem_system_font(repo_root, path, f):
    # type: (str, str, IO[bytes]) -> List[rules.Error]
    if not path.endswith((".html", ".htm", ".xht", ".xhtml")):
        return []
    contents = f.read()
    errors = []
    if ahem_font_re.search(contents) and not ahem_stylesheet_re.search(contents):
        errors.append(rules.AhemSystemFont.error(path))
    return errors


def check_path(repo_root, path):
    # type: (str, str) -> List[rules.Error]
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
    # type: (str, List[str]) -> List[rules.Error]
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
    # type: (str, str, IO[bytes]) -> List[rules.Error]
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
    # type: (List[rules.Error]) -> None
    assert logger is not None
    for error_type, description, path, line_number in errors:
        pos_string = path
        if line_number:
            pos_string += ":%s" % line_number
        logger.error("%s: %s (%s)" % (pos_string, description, error_type))


def output_errors_markdown(errors):
    # type: (List[rules.Error]) -> None
    if not errors:
        return
    assert logger is not None
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
    # type: (List[rules.Error]) -> None
    for error_type, error, path, line_number in errors:
        print(json.dumps({"path": path, "lineno": line_number,
                          "rule": error_type, "message": error}))


def output_error_count(error_count):
    # type: (Dict[Text, int]) -> None
    if not error_count:
        return

    assert logger is not None
    by_type = " ".join("%s: %d" % item for item in error_count.items())
    count = sum(error_count.values())
    logger.info("")
    if count == 1:
        logger.info("There was 1 error (%s)" % (by_type,))
    else:
        logger.info("There were %d errors (%s)" % (count, by_type))


def changed_files(wpt_root):
    # type: (str) -> List[Text]
    revish = testfiles.get_revish(revish=None)
    changed, _ = testfiles.files_changed(revish, None, include_uncommitted=True, include_new=True)
    return [os.path.relpath(item, wpt_root) for item in changed]


def lint_paths(kwargs, wpt_root):
    # type: (Dict[str, Any], str) -> List[str]
    if kwargs.get(str("paths")):
        paths = []
        for path in kwargs.get(str("paths"), []):
            if os.path.isdir(path):
                path_dir = list(all_filesystem_paths(wpt_root, path))
                paths.extend(path_dir)
            elif os.path.isfile(path):
                paths.append(os.path.relpath(os.path.abspath(path), wpt_root))


    elif kwargs[str("all")]:
        paths = list(all_filesystem_paths(wpt_root))
    else:
        changed_paths = changed_files(wpt_root)
        force_all = False
        for path in changed_paths:
            path = path.replace(os.path.sep, "/")
            if path == "lint.whitelist" or path.startswith("tools/lint/"):
                force_all = True
                break
        paths = (list(changed_paths) if not force_all  # type: ignore
                 else list(all_filesystem_paths(wpt_root)))

    return paths


def create_parser():
    # type: () -> argparse.ArgumentParser
    parser = argparse.ArgumentParser()
    parser.add_argument("paths", nargs="*",
                        help="List of paths to lint")
    parser.add_argument("--json", action="store_true",
                        help="Output machine-readable JSON format")
    parser.add_argument("--markdown", action="store_true",
                        help="Output markdown")
    parser.add_argument("--repo-root", help="The WPT directory. Use this "
                        "option if the lint script exists outside the repository")
    parser.add_argument("--ignore-glob", help="Additional file glob to ignore.")
    parser.add_argument("--all", action="store_true", help="If no paths are passed, try to lint the whole "
                        "working directory, not just files that changed")
    return parser


def main(**kwargs):
    # type: (**Any) -> int
    assert logger is not None
    if kwargs.get(str("json")) and kwargs.get(str("markdown")):
        logger.critical("Cannot specify --json and --markdown")
        sys.exit(2)

    repo_root = kwargs.get(str('repo_root')) or localpaths.repo_root
    output_format = {(True, False): str("json"),
                     (False, True): str("markdown"),
                     (False, False): str("normal")}[(kwargs.get(str("json"), False),
                                                     kwargs.get(str("markdown"), False))]

    if output_format == "markdown":
        setup_logging(True)

    paths = lint_paths(kwargs, repo_root)

    ignore_glob = kwargs.get(str("ignore_glob")) or str()

    return lint(repo_root, paths, output_format, str(ignore_glob))


def lint(repo_root, paths, output_format, ignore_glob=str()):
    # type: (str, List[str], str, str) -> int
    error_count = defaultdict(int)  # type: Dict[Text, int]
    last = None

    with open(os.path.join(repo_root, "lint.whitelist")) as f:
        whitelist, ignored_files = parse_whitelist(f)

    if ignore_glob:
        ignored_files.add(ignore_glob)

    output_errors = {"json": output_errors_json,
                     "markdown": output_errors_markdown,
                     "normal": output_errors_text}[output_format]

    def process_errors(errors):
        # type: (List[rules.Error]) -> Optional[Tuple[Text, Text]]
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
            assert last is not None
            assert logger is not None
            for line in (ERROR_MSG % (last[0], last[1], last[0], last[1])).split("\n"):
                logger.info(line)
    return sum(itervalues(error_count))

path_lints = [check_file_type, check_path_length, check_worker_collision, check_ahem_copy,
              check_gitignore_file]
all_paths_lints = [check_css_globally_unique]
file_lints = [check_regexp_line, check_parsed, check_python_ast, check_script_metadata,
              check_ahem_system_font]

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
