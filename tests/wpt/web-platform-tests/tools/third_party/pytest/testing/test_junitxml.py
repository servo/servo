import os
import platform
from datetime import datetime
from typing import cast
from typing import List
from typing import Tuple
from xml.dom import minidom

import py
import xmlschema

import pytest
from _pytest.compat import TYPE_CHECKING
from _pytest.config import Config
from _pytest.junitxml import bin_xml_escape
from _pytest.junitxml import LogXML
from _pytest.pathlib import Path
from _pytest.reports import BaseReport
from _pytest.reports import TestReport
from _pytest.store import Store


@pytest.fixture(scope="session")
def schema():
    """Return an xmlschema.XMLSchema object for the junit-10.xsd file."""
    fn = Path(__file__).parent / "example_scripts/junit-10.xsd"
    with fn.open() as f:
        return xmlschema.XMLSchema(f)


@pytest.fixture
def run_and_parse(testdir, schema):
    """Fixture that returns a function that can be used to execute pytest and
    return the parsed ``DomNode`` of the root xml node.

    The ``family`` parameter is used to configure the ``junit_family`` of the written report.
    "xunit2" is also automatically validated against the schema.
    """

    def run(*args, family="xunit1"):
        if family:
            args = ("-o", "junit_family=" + family) + args
        xml_path = testdir.tmpdir.join("junit.xml")
        result = testdir.runpytest("--junitxml=%s" % xml_path, *args)
        if family == "xunit2":
            with xml_path.open() as f:
                schema.validate(f)
        xmldoc = minidom.parse(str(xml_path))
        return result, DomNode(xmldoc)

    return run


def assert_attr(node, **kwargs):
    __tracebackhide__ = True

    def nodeval(node, name):
        anode = node.getAttributeNode(name)
        if anode is not None:
            return anode.value

    expected = {name: str(value) for name, value in kwargs.items()}
    on_node = {name: nodeval(node, name) for name in expected}
    assert on_node == expected


class DomNode:
    def __init__(self, dom):
        self.__node = dom

    def __repr__(self):
        return self.__node.toxml()

    def find_first_by_tag(self, tag):
        return self.find_nth_by_tag(tag, 0)

    def _by_tag(self, tag):
        return self.__node.getElementsByTagName(tag)

    @property
    def children(self):
        return [type(self)(x) for x in self.__node.childNodes]

    @property
    def get_unique_child(self):
        children = self.children
        assert len(children) == 1
        return children[0]

    def find_nth_by_tag(self, tag, n):
        items = self._by_tag(tag)
        try:
            nth = items[n]
        except IndexError:
            pass
        else:
            return type(self)(nth)

    def find_by_tag(self, tag):
        t = type(self)
        return [t(x) for x in self.__node.getElementsByTagName(tag)]

    def __getitem__(self, key):
        node = self.__node.getAttributeNode(key)
        if node is not None:
            return node.value

    def assert_attr(self, **kwargs):
        __tracebackhide__ = True
        return assert_attr(self.__node, **kwargs)

    def toxml(self):
        return self.__node.toxml()

    @property
    def text(self):
        return self.__node.childNodes[0].wholeText

    @property
    def tag(self):
        return self.__node.tagName

    @property
    def next_sibling(self):
        return type(self)(self.__node.nextSibling)


parametrize_families = pytest.mark.parametrize("xunit_family", ["xunit1", "xunit2"])


class TestPython:
    @parametrize_families
    def test_summing_simple(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            def test_pass():
                pass
            def test_fail():
                assert 0
            def test_skip():
                pytest.skip("")
            @pytest.mark.xfail
            def test_xfail():
                assert 0
            @pytest.mark.xfail
            def test_xpass():
                assert 1
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(name="pytest", errors=0, failures=1, skipped=2, tests=5)

    @parametrize_families
    def test_summing_simple_with_errors(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            @pytest.fixture
            def fixture():
                raise Exception()
            def test_pass():
                pass
            def test_fail():
                assert 0
            def test_error(fixture):
                pass
            @pytest.mark.xfail
            def test_xfail():
                assert False
            @pytest.mark.xfail(strict=True)
            def test_xpass():
                assert True
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(name="pytest", errors=1, failures=2, skipped=1, tests=5)

    @parametrize_families
    def test_hostname_in_xml(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            def test_pass():
                pass
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(hostname=platform.node())

    @parametrize_families
    def test_timestamp_in_xml(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            def test_pass():
                pass
        """
        )
        start_time = datetime.now()
        result, dom = run_and_parse(family=xunit_family)
        node = dom.find_first_by_tag("testsuite")
        timestamp = datetime.strptime(node["timestamp"], "%Y-%m-%dT%H:%M:%S.%f")
        assert start_time <= timestamp < datetime.now()

    def test_timing_function(self, testdir, run_and_parse, mock_timing):
        testdir.makepyfile(
            """
            from _pytest import timing
            def setup_module():
                timing.sleep(1)
            def teardown_module():
                timing.sleep(2)
            def test_sleep():
                timing.sleep(4)
        """
        )
        result, dom = run_and_parse()
        node = dom.find_first_by_tag("testsuite")
        tnode = node.find_first_by_tag("testcase")
        val = tnode["time"]
        assert float(val) == 7.0

    @pytest.mark.parametrize("duration_report", ["call", "total"])
    def test_junit_duration_report(
        self, testdir, monkeypatch, duration_report, run_and_parse
    ):

        # mock LogXML.node_reporter so it always sets a known duration to each test report object
        original_node_reporter = LogXML.node_reporter

        def node_reporter_wrapper(s, report):
            report.duration = 1.0
            reporter = original_node_reporter(s, report)
            return reporter

        monkeypatch.setattr(LogXML, "node_reporter", node_reporter_wrapper)

        testdir.makepyfile(
            """
            def test_foo():
                pass
        """
        )
        result, dom = run_and_parse(
            "-o", "junit_duration_report={}".format(duration_report)
        )
        node = dom.find_first_by_tag("testsuite")
        tnode = node.find_first_by_tag("testcase")
        val = float(tnode["time"])
        if duration_report == "total":
            assert val == 3.0
        else:
            assert duration_report == "call"
            assert val == 1.0

    @parametrize_families
    def test_setup_error(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture
            def arg(request):
                raise ValueError("Error reason")
            def test_function(arg):
                pass
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=1, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_setup_error", name="test_function")
        fnode = tnode.find_first_by_tag("error")
        fnode.assert_attr(message='failed on setup with "ValueError: Error reason"')
        assert "ValueError" in fnode.toxml()

    @parametrize_families
    def test_teardown_error(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture
            def arg():
                yield
                raise ValueError('Error reason')
            def test_function(arg):
                pass
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_teardown_error", name="test_function")
        fnode = tnode.find_first_by_tag("error")
        fnode.assert_attr(message='failed on teardown with "ValueError: Error reason"')
        assert "ValueError" in fnode.toxml()

    @parametrize_families
    def test_call_failure_teardown_error(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture
            def arg():
                yield
                raise Exception("Teardown Exception")
            def test_function(arg):
                raise Exception("Call Exception")
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=1, failures=1, tests=1)
        first, second = dom.find_by_tag("testcase")
        assert first
        assert second
        assert first != second
        fnode = first.find_first_by_tag("failure")
        fnode.assert_attr(message="Exception: Call Exception")
        snode = second.find_first_by_tag("error")
        snode.assert_attr(
            message='failed on teardown with "Exception: Teardown Exception"'
        )

    @parametrize_families
    def test_skip_contains_name_reason(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            def test_skip():
                pytest.skip("hello23")
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret == 0
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skipped=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_skip_contains_name_reason", name="test_skip")
        snode = tnode.find_first_by_tag("skipped")
        snode.assert_attr(type="pytest.skip", message="hello23")

    @parametrize_families
    def test_mark_skip_contains_name_reason(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip(reason="hello24")
            def test_skip():
                assert True
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret == 0
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skipped=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            classname="test_mark_skip_contains_name_reason", name="test_skip"
        )
        snode = tnode.find_first_by_tag("skipped")
        snode.assert_attr(type="pytest.skip", message="hello24")

    @parametrize_families
    def test_mark_skipif_contains_name_reason(
        self, testdir, run_and_parse, xunit_family
    ):
        testdir.makepyfile(
            """
            import pytest
            GLOBAL_CONDITION = True
            @pytest.mark.skipif(GLOBAL_CONDITION, reason="hello25")
            def test_skip():
                assert True
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret == 0
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skipped=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            classname="test_mark_skipif_contains_name_reason", name="test_skip"
        )
        snode = tnode.find_first_by_tag("skipped")
        snode.assert_attr(type="pytest.skip", message="hello25")

    @parametrize_families
    def test_mark_skip_doesnt_capture_output(
        self, testdir, run_and_parse, xunit_family
    ):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.skip(reason="foo")
            def test_skip():
                print("bar!")
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret == 0
        node_xml = dom.find_first_by_tag("testsuite").toxml()
        assert "bar!" not in node_xml

    @parametrize_families
    def test_classname_instance(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            class TestClass(object):
                def test_method(self):
                    assert 0
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            classname="test_classname_instance.TestClass", name="test_method"
        )

    @parametrize_families
    def test_classname_nested_dir(self, testdir, run_and_parse, xunit_family):
        p = testdir.tmpdir.ensure("sub", "test_hello.py")
        p.write("def test_func(): 0/0")
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="sub.test_hello", name="test_func")

    @parametrize_families
    def test_internal_error(self, testdir, run_and_parse, xunit_family):
        testdir.makeconftest("def pytest_runtest_protocol(): 0 / 0")
        testdir.makepyfile("def test_function(): pass")
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=1, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="pytest", name="internal")
        fnode = tnode.find_first_by_tag("error")
        fnode.assert_attr(message="internal error")
        assert "Division" in fnode.toxml()

    @pytest.mark.parametrize(
        "junit_logging", ["no", "log", "system-out", "system-err", "out-err", "all"]
    )
    @parametrize_families
    def test_failure_function(
        self, testdir, junit_logging, run_and_parse, xunit_family
    ):
        testdir.makepyfile(
            """
            import logging
            import sys

            def test_fail():
                print("hello-stdout")
                sys.stderr.write("hello-stderr\\n")
                logging.info('info msg')
                logging.warning('warning msg')
                raise ValueError(42)
        """
        )

        result, dom = run_and_parse(
            "-o", "junit_logging=%s" % junit_logging, family=xunit_family
        )
        assert result.ret, "Expected ret > 0"
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_failure_function", name="test_fail")
        fnode = tnode.find_first_by_tag("failure")
        fnode.assert_attr(message="ValueError: 42")
        assert "ValueError" in fnode.toxml(), "ValueError not included"

        if junit_logging in ["log", "all"]:
            logdata = tnode.find_first_by_tag("system-out")
            log_xml = logdata.toxml()
            assert logdata.tag == "system-out", "Expected tag: system-out"
            assert "info msg" not in log_xml, "Unexpected INFO message"
            assert "warning msg" in log_xml, "Missing WARN message"
        if junit_logging in ["system-out", "out-err", "all"]:
            systemout = tnode.find_first_by_tag("system-out")
            systemout_xml = systemout.toxml()
            assert systemout.tag == "system-out", "Expected tag: system-out"
            assert "info msg" not in systemout_xml, "INFO message found in system-out"
            assert (
                "hello-stdout" in systemout_xml
            ), "Missing 'hello-stdout' in system-out"
        if junit_logging in ["system-err", "out-err", "all"]:
            systemerr = tnode.find_first_by_tag("system-err")
            systemerr_xml = systemerr.toxml()
            assert systemerr.tag == "system-err", "Expected tag: system-err"
            assert "info msg" not in systemerr_xml, "INFO message found in system-err"
            assert (
                "hello-stderr" in systemerr_xml
            ), "Missing 'hello-stderr' in system-err"
            assert (
                "warning msg" not in systemerr_xml
            ), "WARN message found in system-err"
        if junit_logging == "no":
            assert not tnode.find_by_tag("log"), "Found unexpected content: log"
            assert not tnode.find_by_tag(
                "system-out"
            ), "Found unexpected content: system-out"
            assert not tnode.find_by_tag(
                "system-err"
            ), "Found unexpected content: system-err"

    @parametrize_families
    def test_failure_verbose_message(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import sys
            def test_fail():
                assert 0, "An error"
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        node = dom.find_first_by_tag("testsuite")
        tnode = node.find_first_by_tag("testcase")
        fnode = tnode.find_first_by_tag("failure")
        fnode.assert_attr(message="AssertionError: An error\nassert 0")

    @parametrize_families
    def test_failure_escape(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.parametrize('arg1', "<&'", ids="<&'")
            def test_func(arg1):
                print(arg1)
                assert 0
        """
        )
        result, dom = run_and_parse(
            "-o", "junit_logging=system-out", family=xunit_family
        )
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=3, tests=3)

        for index, char in enumerate("<&'"):

            tnode = node.find_nth_by_tag("testcase", index)
            tnode.assert_attr(
                classname="test_failure_escape", name="test_func[%s]" % char
            )
            sysout = tnode.find_first_by_tag("system-out")
            text = sysout.text
            assert "%s\n" % char in text

    @parametrize_families
    def test_junit_prefixing(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            def test_func():
                assert 0
            class TestHello(object):
                def test_hello(self):
                    pass
        """
        )
        result, dom = run_and_parse("--junitprefix=xyz", family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1, tests=2)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="xyz.test_junit_prefixing", name="test_func")
        tnode = node.find_nth_by_tag("testcase", 1)
        tnode.assert_attr(
            classname="xyz.test_junit_prefixing.TestHello", name="test_hello"
        )

    @parametrize_families
    def test_xfailure_function(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            def test_xfail():
                pytest.xfail("42")
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert not result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skipped=1, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_xfailure_function", name="test_xfail")
        fnode = tnode.find_first_by_tag("skipped")
        fnode.assert_attr(type="pytest.xfail", message="42")

    @parametrize_families
    def test_xfailure_marker(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(reason="42")
            def test_xfail():
                assert False
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        assert not result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skipped=1, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_xfailure_marker", name="test_xfail")
        fnode = tnode.find_first_by_tag("skipped")
        fnode.assert_attr(type="pytest.xfail", message="42")

    @pytest.mark.parametrize(
        "junit_logging", ["no", "log", "system-out", "system-err", "out-err", "all"]
    )
    def test_xfail_captures_output_once(self, testdir, junit_logging, run_and_parse):
        testdir.makepyfile(
            """
            import sys
            import pytest

            @pytest.mark.xfail()
            def test_fail():
                sys.stdout.write('XFAIL This is stdout')
                sys.stderr.write('XFAIL This is stderr')
                assert 0
        """
        )
        result, dom = run_and_parse("-o", "junit_logging=%s" % junit_logging)
        node = dom.find_first_by_tag("testsuite")
        tnode = node.find_first_by_tag("testcase")
        if junit_logging in ["system-err", "out-err", "all"]:
            assert len(tnode.find_by_tag("system-err")) == 1
        else:
            assert len(tnode.find_by_tag("system-err")) == 0

        if junit_logging in ["log", "system-out", "out-err", "all"]:
            assert len(tnode.find_by_tag("system-out")) == 1
        else:
            assert len(tnode.find_by_tag("system-out")) == 0

    @parametrize_families
    def test_xfailure_xpass(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail
            def test_xpass():
                pass
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        # assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skipped=0, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_xfailure_xpass", name="test_xpass")

    @parametrize_families
    def test_xfailure_xpass_strict(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile(
            """
            import pytest
            @pytest.mark.xfail(strict=True, reason="This needs to fail!")
            def test_xpass():
                pass
        """
        )
        result, dom = run_and_parse(family=xunit_family)
        # assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skipped=0, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="test_xfailure_xpass_strict", name="test_xpass")
        fnode = tnode.find_first_by_tag("failure")
        fnode.assert_attr(message="[XPASS(strict)] This needs to fail!")

    @parametrize_families
    def test_collect_error(self, testdir, run_and_parse, xunit_family):
        testdir.makepyfile("syntax error")
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=1, tests=1)
        tnode = node.find_first_by_tag("testcase")
        fnode = tnode.find_first_by_tag("error")
        fnode.assert_attr(message="collection failure")
        assert "SyntaxError" in fnode.toxml()

    def test_unicode(self, testdir, run_and_parse):
        value = "hx\xc4\x85\xc4\x87\n"
        testdir.makepyfile(
            """\
            # coding: latin1
            def test_hello():
                print(%r)
                assert 0
            """
            % value
        )
        result, dom = run_and_parse()
        assert result.ret == 1
        tnode = dom.find_first_by_tag("testcase")
        fnode = tnode.find_first_by_tag("failure")
        assert "hx" in fnode.toxml()

    def test_assertion_binchars(self, testdir, run_and_parse):
        """This test did fail when the escaping wasn't strict."""
        testdir.makepyfile(
            """

            M1 = '\x01\x02\x03\x04'
            M2 = '\x01\x02\x03\x05'

            def test_str_compare():
                assert M1 == M2
            """
        )
        result, dom = run_and_parse()
        print(dom.toxml())

    @pytest.mark.parametrize("junit_logging", ["no", "system-out"])
    def test_pass_captures_stdout(self, testdir, run_and_parse, junit_logging):
        testdir.makepyfile(
            """
            def test_pass():
                print('hello-stdout')
        """
        )
        result, dom = run_and_parse("-o", "junit_logging=%s" % junit_logging)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        if junit_logging == "no":
            assert not node.find_by_tag(
                "system-out"
            ), "system-out should not be generated"
        if junit_logging == "system-out":
            systemout = pnode.find_first_by_tag("system-out")
            assert (
                "hello-stdout" in systemout.toxml()
            ), "'hello-stdout' should be in system-out"

    @pytest.mark.parametrize("junit_logging", ["no", "system-err"])
    def test_pass_captures_stderr(self, testdir, run_and_parse, junit_logging):
        testdir.makepyfile(
            """
            import sys
            def test_pass():
                sys.stderr.write('hello-stderr')
        """
        )
        result, dom = run_and_parse("-o", "junit_logging=%s" % junit_logging)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        if junit_logging == "no":
            assert not node.find_by_tag(
                "system-err"
            ), "system-err should not be generated"
        if junit_logging == "system-err":
            systemerr = pnode.find_first_by_tag("system-err")
            assert (
                "hello-stderr" in systemerr.toxml()
            ), "'hello-stderr' should be in system-err"

    @pytest.mark.parametrize("junit_logging", ["no", "system-out"])
    def test_setup_error_captures_stdout(self, testdir, run_and_parse, junit_logging):
        testdir.makepyfile(
            """
            import pytest

            @pytest.fixture
            def arg(request):
                print('hello-stdout')
                raise ValueError()
            def test_function(arg):
                pass
        """
        )
        result, dom = run_and_parse("-o", "junit_logging=%s" % junit_logging)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        if junit_logging == "no":
            assert not node.find_by_tag(
                "system-out"
            ), "system-out should not be generated"
        if junit_logging == "system-out":
            systemout = pnode.find_first_by_tag("system-out")
            assert (
                "hello-stdout" in systemout.toxml()
            ), "'hello-stdout' should be in system-out"

    @pytest.mark.parametrize("junit_logging", ["no", "system-err"])
    def test_setup_error_captures_stderr(self, testdir, run_and_parse, junit_logging):
        testdir.makepyfile(
            """
            import sys
            import pytest

            @pytest.fixture
            def arg(request):
                sys.stderr.write('hello-stderr')
                raise ValueError()
            def test_function(arg):
                pass
        """
        )
        result, dom = run_and_parse("-o", "junit_logging=%s" % junit_logging)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        if junit_logging == "no":
            assert not node.find_by_tag(
                "system-err"
            ), "system-err should not be generated"
        if junit_logging == "system-err":
            systemerr = pnode.find_first_by_tag("system-err")
            assert (
                "hello-stderr" in systemerr.toxml()
            ), "'hello-stderr' should be in system-err"

    @pytest.mark.parametrize("junit_logging", ["no", "system-out"])
    def test_avoid_double_stdout(self, testdir, run_and_parse, junit_logging):
        testdir.makepyfile(
            """
            import sys
            import pytest

            @pytest.fixture
            def arg(request):
                yield
                sys.stdout.write('hello-stdout teardown')
                raise ValueError()
            def test_function(arg):
                sys.stdout.write('hello-stdout call')
        """
        )
        result, dom = run_and_parse("-o", "junit_logging=%s" % junit_logging)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        if junit_logging == "no":
            assert not node.find_by_tag(
                "system-out"
            ), "system-out should not be generated"
        if junit_logging == "system-out":
            systemout = pnode.find_first_by_tag("system-out")
            assert "hello-stdout call" in systemout.toxml()
            assert "hello-stdout teardown" in systemout.toxml()


def test_mangle_test_address():
    from _pytest.junitxml import mangle_test_address

    address = "::".join(["a/my.py.thing.py", "Class", "()", "method", "[a-1-::]"])
    newnames = mangle_test_address(address)
    assert newnames == ["a.my.py.thing", "Class", "method", "[a-1-::]"]


def test_dont_configure_on_workers(tmpdir) -> None:
    gotten = []  # type: List[object]

    class FakeConfig:
        if TYPE_CHECKING:
            workerinput = None

        def __init__(self):
            self.pluginmanager = self
            self.option = self
            self._store = Store()

        def getini(self, name):
            return "pytest"

        junitprefix = None
        # XXX: shouldn't need tmpdir ?
        xmlpath = str(tmpdir.join("junix.xml"))
        register = gotten.append

    fake_config = cast(Config, FakeConfig())
    from _pytest import junitxml

    junitxml.pytest_configure(fake_config)
    assert len(gotten) == 1
    FakeConfig.workerinput = None
    junitxml.pytest_configure(fake_config)
    assert len(gotten) == 1


class TestNonPython:
    @parametrize_families
    def test_summing_simple(self, testdir, run_and_parse, xunit_family):
        testdir.makeconftest(
            """
            import pytest
            def pytest_collect_file(path, parent):
                if path.ext == ".xyz":
                    return MyItem.from_parent(name=path.basename, parent=parent)
            class MyItem(pytest.Item):
                def runtest(self):
                    raise ValueError(42)
                def repr_failure(self, excinfo):
                    return "custom item runtest failed"
        """
        )
        testdir.tmpdir.join("myfile.xyz").write("hello")
        result, dom = run_and_parse(family=xunit_family)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=0, failures=1, skipped=0, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(name="myfile.xyz")
        fnode = tnode.find_first_by_tag("failure")
        fnode.assert_attr(message="custom item runtest failed")
        assert "custom item runtest failed" in fnode.toxml()


@pytest.mark.parametrize("junit_logging", ["no", "system-out"])
def test_nullbyte(testdir, junit_logging):
    # A null byte can not occur in XML (see section 2.2 of the spec)
    testdir.makepyfile(
        """
        import sys
        def test_print_nullbyte():
            sys.stdout.write('Here the null -->' + chr(0) + '<--')
            sys.stdout.write('In repr form -->' + repr(chr(0)) + '<--')
            assert False
    """
    )
    xmlf = testdir.tmpdir.join("junit.xml")
    testdir.runpytest("--junitxml=%s" % xmlf, "-o", "junit_logging=%s" % junit_logging)
    text = xmlf.read()
    assert "\x00" not in text
    if junit_logging == "system-out":
        assert "#x00" in text
    if junit_logging == "no":
        assert "#x00" not in text


@pytest.mark.parametrize("junit_logging", ["no", "system-out"])
def test_nullbyte_replace(testdir, junit_logging):
    # Check if the null byte gets replaced
    testdir.makepyfile(
        """
        import sys
        def test_print_nullbyte():
            sys.stdout.write('Here the null -->' + chr(0) + '<--')
            sys.stdout.write('In repr form -->' + repr(chr(0)) + '<--')
            assert False
    """
    )
    xmlf = testdir.tmpdir.join("junit.xml")
    testdir.runpytest("--junitxml=%s" % xmlf, "-o", "junit_logging=%s" % junit_logging)
    text = xmlf.read()
    if junit_logging == "system-out":
        assert "#x0" in text
    if junit_logging == "no":
        assert "#x0" not in text


def test_invalid_xml_escape():
    # Test some more invalid xml chars, the full range should be
    # tested really but let's just test the edges of the ranges
    # instead.
    # XXX This only tests low unicode character points for now as
    #     there are some issues with the testing infrastructure for
    #     the higher ones.
    # XXX Testing 0xD (\r) is tricky as it overwrites the just written
    #     line in the output, so we skip it too.
    invalid = (
        0x00,
        0x1,
        0xB,
        0xC,
        0xE,
        0x19,
        27,  # issue #126
        0xD800,
        0xDFFF,
        0xFFFE,
        0x0FFFF,
    )  # , 0x110000)
    valid = (0x9, 0xA, 0x20)
    # 0xD, 0xD7FF, 0xE000, 0xFFFD, 0x10000, 0x10FFFF)

    for i in invalid:
        got = bin_xml_escape(chr(i))
        if i <= 0xFF:
            expected = "#x%02X" % i
        else:
            expected = "#x%04X" % i
        assert got == expected
    for i in valid:
        assert chr(i) == bin_xml_escape(chr(i))


def test_logxml_path_expansion(tmpdir, monkeypatch):
    home_tilde = py.path.local(os.path.expanduser("~")).join("test.xml")
    xml_tilde = LogXML("~%stest.xml" % tmpdir.sep, None)
    assert xml_tilde.logfile == home_tilde

    monkeypatch.setenv("HOME", str(tmpdir))
    home_var = os.path.normpath(os.path.expandvars("$HOME/test.xml"))
    xml_var = LogXML("$HOME%stest.xml" % tmpdir.sep, None)
    assert xml_var.logfile == home_var


def test_logxml_changingdir(testdir):
    testdir.makepyfile(
        """
        def test_func():
            import os
            os.chdir("a")
    """
    )
    testdir.tmpdir.mkdir("a")
    result = testdir.runpytest("--junitxml=a/x.xml")
    assert result.ret == 0
    assert testdir.tmpdir.join("a/x.xml").check()


def test_logxml_makedir(testdir):
    """--junitxml should automatically create directories for the xml file"""
    testdir.makepyfile(
        """
        def test_pass():
            pass
    """
    )
    result = testdir.runpytest("--junitxml=path/to/results.xml")
    assert result.ret == 0
    assert testdir.tmpdir.join("path/to/results.xml").check()


def test_logxml_check_isdir(testdir):
    """Give an error if --junit-xml is a directory (#2089)"""
    result = testdir.runpytest("--junit-xml=.")
    result.stderr.fnmatch_lines(["*--junitxml must be a filename*"])


def test_escaped_parametrized_names_xml(testdir, run_and_parse):
    testdir.makepyfile(
        """\
        import pytest
        @pytest.mark.parametrize('char', ["\\x00"])
        def test_func(char):
            assert char
        """
    )
    result, dom = run_and_parse()
    assert result.ret == 0
    node = dom.find_first_by_tag("testcase")
    node.assert_attr(name="test_func[\\x00]")


def test_double_colon_split_function_issue469(testdir, run_and_parse):
    testdir.makepyfile(
        """
        import pytest
        @pytest.mark.parametrize('param', ["double::colon"])
        def test_func(param):
            pass
    """
    )
    result, dom = run_and_parse()
    assert result.ret == 0
    node = dom.find_first_by_tag("testcase")
    node.assert_attr(classname="test_double_colon_split_function_issue469")
    node.assert_attr(name="test_func[double::colon]")


def test_double_colon_split_method_issue469(testdir, run_and_parse):
    testdir.makepyfile(
        """
        import pytest
        class TestClass(object):
            @pytest.mark.parametrize('param', ["double::colon"])
            def test_func(self, param):
                pass
    """
    )
    result, dom = run_and_parse()
    assert result.ret == 0
    node = dom.find_first_by_tag("testcase")
    node.assert_attr(classname="test_double_colon_split_method_issue469.TestClass")
    node.assert_attr(name="test_func[double::colon]")


def test_unicode_issue368(testdir) -> None:
    path = testdir.tmpdir.join("test.xml")
    log = LogXML(str(path), None)
    ustr = "ВНИ!"

    class Report(BaseReport):
        longrepr = ustr
        sections = []  # type: List[Tuple[str, str]]
        nodeid = "something"
        location = "tests/filename.py", 42, "TestClass.method"

    test_report = cast(TestReport, Report())

    # hopefully this is not too brittle ...
    log.pytest_sessionstart()
    node_reporter = log._opentestcase(test_report)
    node_reporter.append_failure(test_report)
    node_reporter.append_collect_error(test_report)
    node_reporter.append_collect_skipped(test_report)
    node_reporter.append_error(test_report)
    test_report.longrepr = "filename", 1, ustr
    node_reporter.append_skipped(test_report)
    test_report.longrepr = "filename", 1, "Skipped: 卡嘣嘣"
    node_reporter.append_skipped(test_report)
    test_report.wasxfail = ustr  # type: ignore[attr-defined]
    node_reporter.append_skipped(test_report)
    log.pytest_sessionfinish()


def test_record_property(testdir, run_and_parse):
    testdir.makepyfile(
        """
        import pytest

        @pytest.fixture
        def other(record_property):
            record_property("bar", 1)
        def test_record(record_property, other):
            record_property("foo", "<1");
    """
    )
    result, dom = run_and_parse()
    node = dom.find_first_by_tag("testsuite")
    tnode = node.find_first_by_tag("testcase")
    psnode = tnode.find_first_by_tag("properties")
    pnodes = psnode.find_by_tag("property")
    pnodes[0].assert_attr(name="bar", value="1")
    pnodes[1].assert_attr(name="foo", value="<1")
    result.stdout.fnmatch_lines(["*= 1 passed in *"])


def test_record_property_same_name(testdir, run_and_parse):
    testdir.makepyfile(
        """
        def test_record_with_same_name(record_property):
            record_property("foo", "bar")
            record_property("foo", "baz")
    """
    )
    result, dom = run_and_parse()
    node = dom.find_first_by_tag("testsuite")
    tnode = node.find_first_by_tag("testcase")
    psnode = tnode.find_first_by_tag("properties")
    pnodes = psnode.find_by_tag("property")
    pnodes[0].assert_attr(name="foo", value="bar")
    pnodes[1].assert_attr(name="foo", value="baz")


@pytest.mark.parametrize("fixture_name", ["record_property", "record_xml_attribute"])
def test_record_fixtures_without_junitxml(testdir, fixture_name):
    testdir.makepyfile(
        """
        def test_record({fixture_name}):
            {fixture_name}("foo", "bar")
    """.format(
            fixture_name=fixture_name
        )
    )
    result = testdir.runpytest()
    assert result.ret == 0


@pytest.mark.filterwarnings("default")
def test_record_attribute(testdir, run_and_parse):
    testdir.makeini(
        """
        [pytest]
        junit_family = xunit1
    """
    )
    testdir.makepyfile(
        """
        import pytest

        @pytest.fixture
        def other(record_xml_attribute):
            record_xml_attribute("bar", 1)
        def test_record(record_xml_attribute, other):
            record_xml_attribute("foo", "<1");
    """
    )
    result, dom = run_and_parse()
    node = dom.find_first_by_tag("testsuite")
    tnode = node.find_first_by_tag("testcase")
    tnode.assert_attr(bar="1")
    tnode.assert_attr(foo="<1")
    result.stdout.fnmatch_lines(
        ["*test_record_attribute.py:6:*record_xml_attribute is an experimental feature"]
    )


@pytest.mark.filterwarnings("default")
@pytest.mark.parametrize("fixture_name", ["record_xml_attribute", "record_property"])
def test_record_fixtures_xunit2(testdir, fixture_name, run_and_parse):
    """Ensure record_xml_attribute and record_property drop values when outside of legacy family."""
    testdir.makeini(
        """
        [pytest]
        junit_family = xunit2
    """
    )
    testdir.makepyfile(
        """
        import pytest

        @pytest.fixture
        def other({fixture_name}):
            {fixture_name}("bar", 1)
        def test_record({fixture_name}, other):
            {fixture_name}("foo", "<1");
    """.format(
            fixture_name=fixture_name
        )
    )

    result, dom = run_and_parse(family=None)
    expected_lines = []
    if fixture_name == "record_xml_attribute":
        expected_lines.append(
            "*test_record_fixtures_xunit2.py:6:*record_xml_attribute is an experimental feature"
        )
    expected_lines = [
        "*test_record_fixtures_xunit2.py:6:*{fixture_name} is incompatible "
        "with junit_family 'xunit2' (use 'legacy' or 'xunit1')".format(
            fixture_name=fixture_name
        )
    ]
    result.stdout.fnmatch_lines(expected_lines)


def test_random_report_log_xdist(testdir, monkeypatch, run_and_parse):
    """`xdist` calls pytest_runtest_logreport as they are executed by the workers,
    with nodes from several nodes overlapping, so junitxml must cope with that
    to produce correct reports (#1064)."""
    pytest.importorskip("xdist")
    monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD", raising=False)
    testdir.makepyfile(
        """
        import pytest, time
        @pytest.mark.parametrize('i', list(range(30)))
        def test_x(i):
            assert i != 22
    """
    )
    _, dom = run_and_parse("-n2")
    suite_node = dom.find_first_by_tag("testsuite")
    failed = []
    for case_node in suite_node.find_by_tag("testcase"):
        if case_node.find_first_by_tag("failure"):
            failed.append(case_node["name"])

    assert failed == ["test_x[22]"]


@parametrize_families
def test_root_testsuites_tag(testdir, run_and_parse, xunit_family):
    testdir.makepyfile(
        """
        def test_x():
            pass
    """
    )
    _, dom = run_and_parse(family=xunit_family)
    root = dom.get_unique_child
    assert root.tag == "testsuites"
    suite_node = root.get_unique_child
    assert suite_node.tag == "testsuite"


def test_runs_twice(testdir, run_and_parse):
    f = testdir.makepyfile(
        """
        def test_pass():
            pass
    """
    )

    result, dom = run_and_parse(f, f)
    result.stdout.no_fnmatch_line("*INTERNALERROR*")
    first, second = [x["classname"] for x in dom.find_by_tag("testcase")]
    assert first == second


def test_runs_twice_xdist(testdir, run_and_parse):
    pytest.importorskip("xdist")
    testdir.monkeypatch.delenv("PYTEST_DISABLE_PLUGIN_AUTOLOAD")
    f = testdir.makepyfile(
        """
        def test_pass():
            pass
    """
    )

    result, dom = run_and_parse(f, "--dist", "each", "--tx", "2*popen")
    result.stdout.no_fnmatch_line("*INTERNALERROR*")
    first, second = [x["classname"] for x in dom.find_by_tag("testcase")]
    assert first == second


def test_fancy_items_regression(testdir, run_and_parse):
    # issue 1259
    testdir.makeconftest(
        """
        import pytest
        class FunItem(pytest.Item):
            def runtest(self):
                pass
        class NoFunItem(pytest.Item):
            def runtest(self):
                pass

        class FunCollector(pytest.File):
            def collect(self):
                return [
                    FunItem.from_parent(name='a', parent=self),
                    NoFunItem.from_parent(name='a', parent=self),
                    NoFunItem.from_parent(name='b', parent=self),
                ]

        def pytest_collect_file(path, parent):
            if path.check(ext='.py'):
                return FunCollector.from_parent(fspath=path, parent=parent)
    """
    )

    testdir.makepyfile(
        """
        def test_pass():
            pass
    """
    )

    result, dom = run_and_parse()

    result.stdout.no_fnmatch_line("*INTERNALERROR*")

    items = sorted("%(classname)s %(name)s" % x for x in dom.find_by_tag("testcase"))
    import pprint

    pprint.pprint(items)
    assert items == [
        "conftest a",
        "conftest a",
        "conftest b",
        "test_fancy_items_regression a",
        "test_fancy_items_regression a",
        "test_fancy_items_regression b",
        "test_fancy_items_regression test_pass",
    ]


@parametrize_families
def test_global_properties(testdir, xunit_family) -> None:
    path = testdir.tmpdir.join("test_global_properties.xml")
    log = LogXML(str(path), None, family=xunit_family)

    class Report(BaseReport):
        sections = []  # type: List[Tuple[str, str]]
        nodeid = "test_node_id"

    log.pytest_sessionstart()
    log.add_global_property("foo", "1")
    log.add_global_property("bar", "2")
    log.pytest_sessionfinish()

    dom = minidom.parse(str(path))

    properties = dom.getElementsByTagName("properties")

    assert properties.length == 1, "There must be one <properties> node"

    property_list = dom.getElementsByTagName("property")

    assert property_list.length == 2, "There most be only 2 property nodes"

    expected = {"foo": "1", "bar": "2"}
    actual = {}

    for p in property_list:
        k = str(p.getAttribute("name"))
        v = str(p.getAttribute("value"))
        actual[k] = v

    assert actual == expected


def test_url_property(testdir) -> None:
    test_url = "http://www.github.com/pytest-dev"
    path = testdir.tmpdir.join("test_url_property.xml")
    log = LogXML(str(path), None)

    class Report(BaseReport):
        longrepr = "FooBarBaz"
        sections = []  # type: List[Tuple[str, str]]
        nodeid = "something"
        location = "tests/filename.py", 42, "TestClass.method"
        url = test_url

    test_report = cast(TestReport, Report())

    log.pytest_sessionstart()
    node_reporter = log._opentestcase(test_report)
    node_reporter.append_failure(test_report)
    log.pytest_sessionfinish()

    test_case = minidom.parse(str(path)).getElementsByTagName("testcase")[0]

    assert (
        test_case.getAttribute("url") == test_url
    ), "The URL did not get written to the xml"


@parametrize_families
def test_record_testsuite_property(testdir, run_and_parse, xunit_family):
    testdir.makepyfile(
        """
        def test_func1(record_testsuite_property):
            record_testsuite_property("stats", "all good")

        def test_func2(record_testsuite_property):
            record_testsuite_property("stats", 10)
    """
    )
    result, dom = run_and_parse(family=xunit_family)
    assert result.ret == 0
    node = dom.find_first_by_tag("testsuite")
    properties_node = node.find_first_by_tag("properties")
    p1_node = properties_node.find_nth_by_tag("property", 0)
    p2_node = properties_node.find_nth_by_tag("property", 1)
    p1_node.assert_attr(name="stats", value="all good")
    p2_node.assert_attr(name="stats", value="10")


def test_record_testsuite_property_junit_disabled(testdir):
    testdir.makepyfile(
        """
        def test_func1(record_testsuite_property):
            record_testsuite_property("stats", "all good")
    """
    )
    result = testdir.runpytest()
    assert result.ret == 0


@pytest.mark.parametrize("junit", [True, False])
def test_record_testsuite_property_type_checking(testdir, junit):
    testdir.makepyfile(
        """
        def test_func1(record_testsuite_property):
            record_testsuite_property(1, 2)
    """
    )
    args = ("--junitxml=tests.xml",) if junit else ()
    result = testdir.runpytest(*args)
    assert result.ret == 1
    result.stdout.fnmatch_lines(
        ["*TypeError: name parameter needs to be a string, but int given"]
    )


@pytest.mark.parametrize("suite_name", ["my_suite", ""])
@parametrize_families
def test_set_suite_name(testdir, suite_name, run_and_parse, xunit_family):
    if suite_name:
        testdir.makeini(
            """
            [pytest]
            junit_suite_name={suite_name}
            junit_family={family}
        """.format(
                suite_name=suite_name, family=xunit_family
            )
        )
        expected = suite_name
    else:
        expected = "pytest"
    testdir.makepyfile(
        """
        import pytest

        def test_func():
            pass
    """
    )
    result, dom = run_and_parse(family=xunit_family)
    assert result.ret == 0
    node = dom.find_first_by_tag("testsuite")
    node.assert_attr(name=expected)


def test_escaped_skipreason_issue3533(testdir, run_and_parse):
    testdir.makepyfile(
        """
        import pytest
        @pytest.mark.skip(reason='1 <> 2')
        def test_skip():
            pass
    """
    )
    _, dom = run_and_parse()
    node = dom.find_first_by_tag("testcase")
    snode = node.find_first_by_tag("skipped")
    assert "1 <> 2" in snode.text
    snode.assert_attr(message="1 <> 2")


@parametrize_families
def test_logging_passing_tests_disabled_does_not_log_test_output(
    testdir, run_and_parse, xunit_family
):
    testdir.makeini(
        """
        [pytest]
        junit_log_passing_tests=False
        junit_logging=system-out
        junit_family={family}
    """.format(
            family=xunit_family
        )
    )
    testdir.makepyfile(
        """
        import pytest
        import logging
        import sys

        def test_func():
            sys.stdout.write('This is stdout')
            sys.stderr.write('This is stderr')
            logging.warning('hello')
    """
    )
    result, dom = run_and_parse(family=xunit_family)
    assert result.ret == 0
    node = dom.find_first_by_tag("testcase")
    assert len(node.find_by_tag("system-err")) == 0
    assert len(node.find_by_tag("system-out")) == 0


@parametrize_families
@pytest.mark.parametrize("junit_logging", ["no", "system-out", "system-err"])
def test_logging_passing_tests_disabled_logs_output_for_failing_test_issue5430(
    testdir, junit_logging, run_and_parse, xunit_family
):
    testdir.makeini(
        """
        [pytest]
        junit_log_passing_tests=False
        junit_family={family}
    """.format(
            family=xunit_family
        )
    )
    testdir.makepyfile(
        """
        import pytest
        import logging
        import sys

        def test_func():
            logging.warning('hello')
            assert 0
    """
    )
    result, dom = run_and_parse(
        "-o", "junit_logging=%s" % junit_logging, family=xunit_family
    )
    assert result.ret == 1
    node = dom.find_first_by_tag("testcase")
    if junit_logging == "system-out":
        assert len(node.find_by_tag("system-err")) == 0
        assert len(node.find_by_tag("system-out")) == 1
    elif junit_logging == "system-err":
        assert len(node.find_by_tag("system-err")) == 1
        assert len(node.find_by_tag("system-out")) == 0
    else:
        assert junit_logging == "no"
        assert len(node.find_by_tag("system-err")) == 0
        assert len(node.find_by_tag("system-out")) == 0
