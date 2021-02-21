import copy
import json
import os
import ssl
import sys
import subprocess

import html5lib
import py
import pytest
from six import text_type
from six.moves import urllib

from wptserver import WPTServer

HERE = os.path.dirname(os.path.abspath(__file__))
WPT_ROOT = os.path.normpath(os.path.join(HERE, '..', '..'))
HARNESS = os.path.join(HERE, 'harness.html')
TEST_TYPES = ('functional', 'unit')
DEFAULT_VARIANTS = ["?default"]

sys.path.insert(0, os.path.normpath(os.path.join(WPT_ROOT, "tools")))
import localpaths

sys.path.insert(0, os.path.normpath(os.path.join(WPT_ROOT, "tools", "webdriver")))
import webdriver


def pytest_addoption(parser):
    parser.addoption("--binary", action="store", default=None, help="path to browser binary")
    parser.addoption("--headless", action="store_true", default=False, help="run browser in headless mode")


def pytest_collect_file(path, parent):
    if path.ext.lower() != '.html':
        return

    # Tests are organized in directories by type
    test_type = os.path.relpath(str(path), HERE)
    if os.path.sep not in test_type or ".." in test_type:
        # HTML files in this directory are not tests
        return
    test_type = test_type.split(os.path.sep)[1]

    # Handle the deprecation of Node construction in pytest6
    # https://docs.pytest.org/en/stable/deprecations.html#node-construction-changed-to-node-from-parent
    if hasattr(HTMLItem, "from_parent"):
        return HTMLItem.from_parent(parent, filename=str(path), test_type=test_type)
    return HTMLItem(parent, str(path), test_type)


def pytest_configure(config):
    config.proc = subprocess.Popen(["geckodriver"])
    config.add_cleanup(config.proc.kill)

    capabilities = {"alwaysMatch": {"acceptInsecureCerts": True, "moz:firefoxOptions": {}}}
    if config.getoption("--binary"):
        capabilities["alwaysMatch"]["moz:firefoxOptions"]["binary"] = config.getoption("--binary")
    if config.getoption("--headless"):
        capabilities["alwaysMatch"]["moz:firefoxOptions"]["args"] = ["--headless"]

    config.driver = webdriver.Session("localhost", 4444,
                                      capabilities=capabilities)
    config.add_cleanup(config.driver.end)

    config.server = WPTServer(WPT_ROOT)
    config.server.start()
    # Although the name of the `_create_unverified_context` method suggests
    # that it is not intended for external consumption, the standard library's
    # documentation explicitly endorses its use:
    #
    # > To revert to the previous, unverified, behavior
    # > ssl._create_unverified_context() can be passed to the context
    # > parameter.
    #
    # https://docs.python.org/2/library/httplib.html#httplib.HTTPSConnection
    config.ssl_context = ssl._create_unverified_context()
    config.add_cleanup(config.server.stop)


def resolve_uri(context, uri):
    if uri.startswith('/'):
        base = WPT_ROOT
        path = uri[1:]
    else:
        base = os.path.dirname(context)
        path = uri

    return os.path.exists(os.path.join(base, path))


class HTMLItem(pytest.Item, pytest.Collector):
    def __init__(self, parent, filename, test_type):
        self.url = parent.session.config.server.url(filename)
        self.type = test_type
        self.variants = []
        # Some tests are reliant on the WPT servers substitution functionality,
        # so tests must be retrieved from the server rather than read from the
        # file system directly.
        handle = urllib.request.urlopen(self.url,
                                        context=parent.session.config.ssl_context)
        try:
            markup = handle.read()
        finally:
            handle.close()

        if test_type not in TEST_TYPES:
            raise ValueError('Unrecognized test type: "%s"' % test_type)

        parsed = html5lib.parse(markup, namespaceHTMLElements=False)
        name = None
        includes_variants_script = False
        self.expected = None

        for element in parsed.getiterator():
            if not name and element.tag == 'title':
                name = element.text
                continue
            if element.tag == 'meta' and element.attrib.get('name') == 'variant':
                self.variants.append(element.attrib.get('content'))
                continue
            if element.tag == 'script':
                if element.attrib.get('id') == 'expected':
                    try:
                        self.expected = json.loads(text_type(element.text))
                    except ValueError:
                        print("Failed parsing JSON in %s" % filename)
                        raise

                src = element.attrib.get('src', '')

                if 'variants.js' in src:
                    includes_variants_script = True
                    if not resolve_uri(filename, src):
                        raise ValueError('Could not resolve path "%s" from %s' % (src, filename))

        if not name:
            raise ValueError('No name found in %s add a <title> element' % filename)
        elif self.type == 'functional':
            if not self.expected:
                raise ValueError('Functional tests must specify expected report data')
            if not includes_variants_script:
                raise ValueError('No variants script found in file %s add '
                                 '\'<script src="../../variants.js"></script>\'' % filename)
            if len(self.variants) == 0:
                self.variants = DEFAULT_VARIANTS
        elif self.type == 'unit' and self.expected:
            raise ValueError('Unit tests must not specify expected report data')

        # Ensure that distinct items have distinct fspath attributes.
        # This is necessary because pytest has an internal cache keyed on it,
        # and only the first test with any given fspath will be run.
        #
        # This cannot use super(HTMLItem, self).__init__(..) because only the
        # Collector constructor takes the fspath argument.
        pytest.Item.__init__(self, name, parent)
        pytest.Collector.__init__(self, name, parent, fspath=py.path.local(filename))


    def reportinfo(self):
        return self.fspath, None, self.url

    def repr_failure(self, excinfo):
        return pytest.Collector.repr_failure(self, excinfo)

    def runtest(self):
        if self.type == 'unit':
            self._run_unit_test()
        elif self.type == 'functional':
            self._run_functional_test()
        else:
            raise NotImplementedError

    def _run_unit_test(self):
        driver = self.session.config.driver
        server = self.session.config.server

        driver.url = server.url(HARNESS)

        actual = driver.execute_async_script(
            'runTest("%s", "foo", arguments[0])' % self.url
        )

        summarized = self._summarize(copy.deepcopy(actual))

        print(json.dumps(summarized, indent=2))

        assert summarized[u'summarized_status'][u'status_string'] == u'OK', summarized[u'summarized_status'][u'message']
        for test in summarized[u'summarized_tests']:
            msg = "%s\n%s" % (test[u'name'], test[u'message'])
            assert test[u'status_string'] == u'PASS', msg

    def _run_functional_test(self):
        for variant in self.variants:
            self._run_functional_test_variant(variant)

    def _run_functional_test_variant(self, variant):
        driver = self.session.config.driver
        server = self.session.config.server

        driver.url = server.url(HARNESS)

        test_url = self.url + variant
        actual = driver.execute_async_script('runTest("%s", "foo", arguments[0])' % test_url)

        print(json.dumps(actual, indent=2))

        summarized = self._summarize(copy.deepcopy(actual))

        print(json.dumps(summarized, indent=2))

        # Test object ordering is not guaranteed. This weak assertion verifies
        # that the indices are unique and sequential
        indices = [test_obj.get('index') for test_obj in actual['tests']]
        self._assert_sequence(indices)

        self.expected[u'summarized_tests'].sort(key=lambda test_obj: test_obj.get('name'))

        # Make asserts opt-in for now
        if "summarized_asserts" not in self.expected:
            del summarized["summarized_asserts"]
        else:
            # We can't be sure of the order of asserts even within the same test
            # although we could also check for the failing assert being the final
            # one
            for obj in [summarized, self.expected]:
                obj["summarized_asserts"].sort(
                    key=lambda x: (x["test"] or "", x["status"], x["assert_name"], tuple(x["args"])))

        assert summarized == self.expected

    def _summarize(self, actual):
        summarized = {}

        summarized[u'summarized_status'] = self._summarize_status(actual['status'])
        summarized[u'summarized_tests'] = [
            self._summarize_test(test) for test in actual['tests']]
        summarized[u'summarized_tests'].sort(key=lambda test_obj: test_obj.get('name'))
        summarized[u'summarized_asserts'] = [
            {"assert_name": assert_item["assert_name"],
            "test": assert_item["test"]["name"] if assert_item["test"] else None,
            "args": assert_item["args"],
            "status": assert_item["status"]} for assert_item in actual["asserts"]]
        summarized[u'type'] = actual['type']

        return summarized

    @staticmethod
    def _assert_sequence(nums):
        if nums and len(nums) > 0:
            assert nums == list(range(1, nums[-1] + 1))

    @staticmethod
    def _scrub_stack(test_obj):
        copy = dict(test_obj)
        del copy['stack']
        return copy

    @staticmethod
    def _expand_status(status_obj):
        for key, value in [item for item in status_obj.items()]:
            # In "status" and "test" objects, the "status" value enum
            # definitions are interspersed with properties for unrelated
            # metadata. The following condition is a best-effort attempt to
            # ignore non-enum properties.
            if key != key.upper() or not isinstance(value, int):
                continue

            del status_obj[key]

            if status_obj['status'] == value:
                status_obj[u'status_string'] = key

        del status_obj['status']

        return status_obj

    @staticmethod
    def _summarize_test(test_obj):
        del test_obj['index']

        assert 'phase' in test_obj
        assert 'phases' in test_obj
        assert 'COMPLETE' in test_obj['phases']
        assert test_obj['phase'] == test_obj['phases']['COMPLETE']
        del test_obj['phases']
        del test_obj['phase']

        return HTMLItem._expand_status(HTMLItem._scrub_stack(test_obj))

    @staticmethod
    def _summarize_status(status_obj):
        return HTMLItem._expand_status(HTMLItem._scrub_stack(status_obj))
