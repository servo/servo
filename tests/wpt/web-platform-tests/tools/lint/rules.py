from __future__ import unicode_literals

import abc
import os
import re

import six

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Any, List, Match, Optional, Pattern, Text, Tuple, cast
    Error = Tuple[Text, Text, Text, Optional[int]]


class Rule(six.with_metaclass(abc.ABCMeta)):
    @abc.abstractproperty
    def name(self):
        # type: () -> Text
        pass

    @abc.abstractproperty
    def description(self):
        # type: () -> Text
        pass

    to_fix = None  # type: Optional[Text]

    @classmethod
    def error(cls, path, context=(), line_no=None):
        # type: (Text, Tuple[Any, ...], Optional[int]) -> Error
        if MYPY:
            name = cast(Text, cls.name)
            description = cast(Text, cls.description)
        else:
            name = cls.name
            description = cls.description
        description = description % context
        return (name, description, path, line_no)


class MissingLink(Rule):
    name = "MISSING-LINK"
    description = "Testcase file must have a link to a spec"
    to_fix = """
        Ensure that there is a `<link rel="help" href="[url]">` for the spec.
        `MISSING-LINK` is designed to ensure that the CSS build tool can find
        the tests. Note that the CSS build system is primarily used by
        [test.csswg.org/](http://test.csswg.org/), which doesn't use
        `wptserve`, so `*.any.js` and similar tests won't work there; stick
        with the `.html` equivalent.
    """


class PathLength(Rule):
    name = "PATH LENGTH"
    description = "/%s longer than maximum path length (%d > 150)"


class FileType(Rule):
    name = "FILE TYPE"
    description = "/%s is an unsupported file type (%s)"


class WorkerCollision(Rule):
    name = "WORKER COLLISION"
    description = ("path ends with %s which collides with generated tests "
        "from %s files")


class GitIgnoreFile(Rule):
    name = "GITIGNORE"
    description = ".gitignore found outside the root"


class AhemCopy(Rule):
    name = "AHEM COPY"
    description = "Don't add extra copies of Ahem, use /fonts/Ahem.ttf"


class AhemSystemFont(Rule):
    name = "AHEM SYSTEM FONT"
    description = "Don't use Ahem as a system font, use /fonts/ahem.css"


# TODO: Add tests for this rule
class IgnoredPath(Rule):
    name = "IGNORED PATH"
    description = ("%s matches an ignore filter in .gitignore - "
        "please add a .gitignore exception")


class CSSCollidingTestName(Rule):
    name = "CSS-COLLIDING-TEST-NAME"
    description = "The filename %s in the %s testsuite is shared by: %s"


class CSSCollidingRefName(Rule):
    name = "CSS-COLLIDING-REF-NAME"
    description = "The filename %s is shared by: %s"


class CSSCollidingSupportName(Rule):
    name = "CSS-COLLIDING-SUPPORT-NAME"
    description = "The filename %s is shared by: %s"


class SupportWrongDir(Rule):
    name = "SUPPORT-WRONG-DIR"
    description = "Support file not in support directory"


class ParseFailed(Rule):
    name = "PARSE-FAILED"
    description = "Unable to parse file"


class ContentManual(Rule):
    name = "CONTENT-MANUAL"
    description = "Manual test whose filename doesn't end in '-manual'"


class ContentVisual(Rule):
    name = "CONTENT-VISUAL"
    description = "Visual test whose filename doesn't end in '-visual'"


class AbsoluteUrlRef(Rule):
    name = "ABSOLUTE-URL-REF"
    description = ("Reference test with a reference file specified via an "
        "absolute URL: '%s'")


class SameFileRef(Rule):
    name = "SAME-FILE-REF"
    description = "Reference test which points at itself as a reference"


class NonexistentRef(Rule):
    name = "NON-EXISTENT-REF"
    description = ("Reference test with a non-existent '%s' relationship "
        "reference: '%s'")


class MultipleTimeout(Rule):
    name = "MULTIPLE-TIMEOUT"
    description = "More than one meta name='timeout'"


class InvalidTimeout(Rule):
    name = "INVALID-TIMEOUT"
    description = "Invalid timeout value %s"


class MultipleTestharness(Rule):
    name = "MULTIPLE-TESTHARNESS"
    description = "More than one <script src='/resources/testharness.js'>"


class MissingTestharnessReport(Rule):
    name = "MISSING-TESTHARNESSREPORT"
    description = "Missing <script src='/resources/testharnessreport.js'>"


class MultipleTestharnessReport(Rule):
    name = "MULTIPLE-TESTHARNESSREPORT"
    description = "More than one <script src='/resources/testharnessreport.js'>"


class PresentTestharnessCSS(Rule):
    name = "PRESENT-TESTHARNESSCSS"
    description = "Explicit link to testharness.css present"


class VariantMissing(Rule):
    name = "VARIANT-MISSING"
    description = "<meta name=variant> missing 'content' attribute"


class MalformedVariant(Rule):
    name = "MALFORMED-VARIANT"
    description = ("%s <meta name=variant> 'content' attribute must be the "
        "empty string or start with '?' or '#'")


class LateTimeout(Rule):
    name = "LATE-TIMEOUT"
    description = "<meta name=timeout> seen after testharness.js script"


class EarlyTestharnessReport(Rule):
    name = "EARLY-TESTHARNESSREPORT"
    description = "testharnessreport.js script seen before testharness.js script"


class MultipleTestdriver(Rule):
    name = "MULTIPLE-TESTDRIVER"
    description = "More than one <script src='/resources/testdriver.js'>"


class MissingTestdriverVendor(Rule):
    name = "MISSING-TESTDRIVER-VENDOR"
    description = "Missing <script src='/resources/testdriver-vendor.js'>"


class MultipleTestdriverVendor(Rule):
    name = "MULTIPLE-TESTDRIVER-VENDOR"
    description = "More than one <script src='/resources/testdriver-vendor.js'>"


class TestharnessPath(Rule):
    name = "TESTHARNESS-PATH"
    description = "testharness.js script seen with incorrect path"


class TestharnessReportPath(Rule):
    name = "TESTHARNESSREPORT-PATH"
    description = "testharnessreport.js script seen with incorrect path"


class TestdriverPath(Rule):
    name = "TESTDRIVER-PATH"
    description = "testdriver.js script seen with incorrect path"


class TestdriverVendorPath(Rule):
    name = "TESTDRIVER-VENDOR-PATH"
    description = "testdriver-vendor.js script seen with incorrect path"


class OpenNoMode(Rule):
    name = "OPEN-NO-MODE"
    description = "File opened without providing an explicit mode (note: binary files must be read with 'b' in the mode flags)"


class UnknownGlobalMetadata(Rule):
    name = "UNKNOWN-GLOBAL-METADATA"
    description = "Unexpected value for global metadata"


class BrokenGlobalMetadata(Rule):
    name = "BROKEN-GLOBAL-METADATA"
    description = "Invalid global metadata: %s"


class UnknownTimeoutMetadata(Rule):
    name = "UNKNOWN-TIMEOUT-METADATA"
    description = "Unexpected value for timeout metadata"


class UnknownMetadata(Rule):
    name = "UNKNOWN-METADATA"
    description = "Unexpected kind of metadata"


class StrayMetadata(Rule):
    name = "STRAY-METADATA"
    description = "Metadata comments should start the file"


class IndentedMetadata(Rule):
    name = "INDENTED-METADATA"
    description = "Metadata comments should start the line"


class BrokenMetadata(Rule):
    name = "BROKEN-METADATA"
    description = "Metadata comment is not formatted correctly"


class Regexp(six.with_metaclass(abc.ABCMeta)):
    @abc.abstractproperty
    def pattern(self):
        # type: () -> bytes
        pass

    @abc.abstractproperty
    def name(self):
        # type: () -> Text
        pass

    @abc.abstractproperty
    def description(self):
        # type: () -> Text
        pass

    file_extensions = None  # type: Optional[List[Text]]

    def __init__(self):
        # type: () -> None
        self._re = re.compile(self.pattern)  # type: Pattern[bytes]

    def applies(self, path):
        # type: (str) -> bool
        return (self.file_extensions is None or
                os.path.splitext(path)[1] in self.file_extensions)

    def search(self, line):
        # type: (bytes) -> Optional[Match[bytes]]
        return self._re.search(line)


class TabsRegexp(Regexp):
    pattern = b"^\t"
    name = "INDENT TABS"
    description = "Tabs used for indentation"

class CRRegexp(Regexp):
    pattern = b"\r$"
    name = "CR AT EOL"
    description = "CR character in line separator"

class SetTimeoutRegexp(Regexp):
    pattern = br"setTimeout\s*\("
    name = "SET TIMEOUT"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "setTimeout used; step_timeout should typically be used instead"

class W3CTestOrgRegexp(Regexp):
    pattern = br"w3c\-test\.org"
    name = "W3C-TEST.ORG"
    description = "External w3c-test.org domain used"

class WebPlatformTestRegexp(Regexp):
    pattern = br"web\-platform\.test"
    name = "WEB-PLATFORM.TEST"
    description = "Internal web-platform.test domain used"

class Webidl2Regexp(Regexp):
    pattern = br"webidl2\.js"
    name = "WEBIDL2.JS"
    description = "Legacy webidl2.js script used"

class ConsoleRegexp(Regexp):
    pattern = br"console\.[a-zA-Z]+\s*\("
    name = "CONSOLE"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "Console logging API used"

class GenerateTestsRegexp(Regexp):
    pattern = br"generate_tests\s*\("
    name = "GENERATE_TESTS"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "generate_tests used"

class PrintRegexp(Regexp):
    pattern = br"print(?:\s|\s*\()"
    name = "PRINT STATEMENT"
    file_extensions = [".py"]
    description = "Print function used"

class LayoutTestsRegexp(Regexp):
    pattern = br"(eventSender|testRunner|internals)\."
    name = "LAYOUTTESTS APIS"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "eventSender/testRunner/internals used; these are LayoutTests-specific APIs (WebKit/Blink)"

class MissingDepsRegexp(Regexp):
    pattern = br"[^\w]/gen/"
    name = "MISSING DEPENDENCY"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "Chromium-specific content referenced"
    to_fix = "Reimplement the test to use well-documented testing interfaces"

class SpecialPowersRegexp(Regexp):
    pattern = b"SpecialPowers"
    name = "SPECIALPOWERS API"
    file_extensions = [".html", ".htm", ".js", ".xht", ".xhtml", ".svg"]
    description = "SpecialPowers used; this is gecko-specific and not supported in wpt"

class TrailingWhitespaceRegexp(Regexp):
    name = "TRAILING WHITESPACE"
    description = "Whitespace at EOL"
    pattern = b"[ \t\f\v]$"
    to_fix = """Remove trailing whitespace from all lines in the file."""
