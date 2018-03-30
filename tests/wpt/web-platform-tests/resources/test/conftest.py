import io
import json
import os

import html5lib
import pytest
from selenium import webdriver

from wptserver import WPTServer

ENC = 'utf8'
HERE = os.path.dirname(os.path.abspath(__file__))
WPT_ROOT = os.path.normpath(os.path.join(HERE, '..', '..'))
HARNESS = os.path.join(HERE, 'harness.html')

def pytest_addoption(parser):
    parser.addoption("--binary", action="store", default=None, help="path to browser binary")

def pytest_collect_file(path, parent):
    if path.ext.lower() == '.html':
        return HTMLItem(str(path), parent)

def pytest_configure(config):
    config.driver = webdriver.Firefox(firefox_binary=config.getoption("--binary"))
    config.server = WPTServer(WPT_ROOT)
    config.server.start()
    config.add_cleanup(config.server.stop)
    config.add_cleanup(config.driver.quit)

class HTMLItem(pytest.Item, pytest.Collector):
    def __init__(self, filename, parent):
        self.filename = filename
        with io.open(filename, encoding=ENC) as f:
            markup = f.read()

        parsed = html5lib.parse(markup, namespaceHTMLElements=False)
        name = None
        self.expected = None

        for element in parsed.getiterator():
            if not name and element.tag == 'title':
                name = element.text
                continue
            if element.attrib.get('id') == 'expected':
                self.expected = json.loads(unicode(element.text))
                continue

        if not name:
            raise ValueError('No name found in file: %s' % filename)

        super(HTMLItem, self).__init__(name, parent)


    def reportinfo(self):
        return self.fspath, None, self.filename

    def repr_failure(self, excinfo):
        return pytest.Collector.repr_failure(self, excinfo)

    def runtest(self):
        driver = self.session.config.driver
        server = self.session.config.server

        driver.get(server.url(HARNESS))

        actual = driver.execute_async_script('runTest("%s", "foo", arguments[0])' % server.url(str(self.filename)))

        # Test object ordering is not guaranteed. This weak assertion verifies
        # that the indices are unique and sequential
        indices = [test_obj.get('index') for test_obj in actual['tests']]
        self._assert_sequence(indices)

        summarized = {}
        summarized[u'summarized_status'] = self._summarize_status(actual['status'])
        summarized[u'summarized_tests'] = [
            self._summarize_test(test) for test in actual['tests']]
        summarized[u'summarized_tests'].sort(key=lambda test_obj: test_obj.get('name'))
        summarized[u'type'] = actual['type']

        if not self.expected:
            assert summarized[u'summarized_status'][u'status_string'] == u'OK', summarized[u'summarized_status'][u'message']
            for test in summarized[u'summarized_tests']:
                msg = "%s\n%s:\n%s" % (test[u'name'], test[u'message'], test[u'stack'])
                assert test[u'status_string'] == u'PASS', msg
        else:
            assert summarized == self.expected

    @staticmethod
    def _assert_sequence(nums):
        assert nums == range(1, nums[-1] + 1)

    @staticmethod
    def _scrub_stack(test_obj):
        copy = dict(test_obj)

        assert 'stack' in copy

        if copy['stack'] is not None:
            copy['stack'] = u'(implementation-defined)'

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
