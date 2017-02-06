# -*- coding: utf-8 -*-

from xml.dom import minidom
from _pytest.main import EXIT_NOTESTSCOLLECTED
import py
import sys
import os
from _pytest.junitxml import LogXML
import pytest


def runandparse(testdir, *args):
    resultpath = testdir.tmpdir.join("junit.xml")
    result = testdir.runpytest("--junitxml=%s" % resultpath, *args)
    xmldoc = minidom.parse(str(resultpath))
    return result, DomNode(xmldoc)


def assert_attr(node, **kwargs):
    __tracebackhide__ = True

    def nodeval(node, name):
        anode = node.getAttributeNode(name)
        if anode is not None:
            return anode.value

    expected = dict((name, str(value)) for name, value in kwargs.items())
    on_node = dict((name, nodeval(node, name)) for name in expected)
    assert on_node == expected


class DomNode(object):
    def __init__(self, dom):
        self.__node = dom

    def __repr__(self):
        return self.__node.toxml()

    def find_first_by_tag(self, tag):
        return self.find_nth_by_tag(tag, 0)

    def _by_tag(self, tag):
        return self.__node.getElementsByTagName(tag)

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
    def next_siebling(self):
        return type(self)(self.__node.nextSibling)


class TestPython:
    def test_summing_simple(self, testdir):
        testdir.makepyfile("""
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
        """)
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(name="pytest", errors=0, failures=1, skips=3, tests=2)

    def test_timing_function(self, testdir):
        testdir.makepyfile("""
            import time, pytest
            def setup_module():
                time.sleep(0.01)
            def teardown_module():
                time.sleep(0.01)
            def test_sleep():
                time.sleep(0.01)
        """)
        result, dom = runandparse(testdir)
        node = dom.find_first_by_tag("testsuite")
        tnode = node.find_first_by_tag("testcase")
        val = tnode["time"]
        assert round(float(val), 2) >= 0.03

    def test_setup_error(self, testdir):
        testdir.makepyfile("""
            def pytest_funcarg__arg(request):
                raise ValueError()
            def test_function(arg):
                pass
        """)
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=1, tests=0)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_setup_error.py",
            line="2",
            classname="test_setup_error",
            name="test_function")
        fnode = tnode.find_first_by_tag("error")
        fnode.assert_attr(message="test setup failure")
        assert "ValueError" in fnode.toxml()

    def test_skip_contains_name_reason(self, testdir):
        testdir.makepyfile("""
            import pytest
            def test_skip():
                pytest.skip("hello23")
        """)
        result, dom = runandparse(testdir)
        assert result.ret == 0
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skips=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_skip_contains_name_reason.py",
            line="1",
            classname="test_skip_contains_name_reason",
            name="test_skip")
        snode = tnode.find_first_by_tag("skipped")
        snode.assert_attr(type="pytest.skip", message="hello23", )

    def test_classname_instance(self, testdir):
        testdir.makepyfile("""
            class TestClass:
                def test_method(self):
                    assert 0
        """)
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_classname_instance.py",
            line="1",
            classname="test_classname_instance.TestClass",
            name="test_method")

    def test_classname_nested_dir(self, testdir):
        p = testdir.tmpdir.ensure("sub", "test_hello.py")
        p.write("def test_func(): 0/0")
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file=os.path.join("sub", "test_hello.py"),
            line="0",
            classname="sub.test_hello",
            name="test_func")

    def test_internal_error(self, testdir):
        testdir.makeconftest("def pytest_runtest_protocol(): 0 / 0")
        testdir.makepyfile("def test_function(): pass")
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=1, tests=0)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(classname="pytest", name="internal")
        fnode = tnode.find_first_by_tag("error")
        fnode.assert_attr(message="internal error")
        assert "Division" in fnode.toxml()

    def test_failure_function(self, testdir):
        testdir.makepyfile("""
            import sys
            def test_fail():
                print ("hello-stdout")
                sys.stderr.write("hello-stderr\\n")
                raise ValueError(42)
        """)

        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_failure_function.py",
            line="1",
            classname="test_failure_function",
            name="test_fail")
        fnode = tnode.find_first_by_tag("failure")
        fnode.assert_attr(message="ValueError: 42")
        assert "ValueError" in fnode.toxml()
        systemout = fnode.next_siebling
        assert systemout.tag == "system-out"
        assert "hello-stdout" in systemout.toxml()
        systemerr = systemout.next_siebling
        assert systemerr.tag == "system-err"
        assert "hello-stderr" in systemerr.toxml()

    def test_failure_verbose_message(self, testdir):
        testdir.makepyfile("""
            import sys
            def test_fail():
                assert 0, "An error"
        """)

        result, dom = runandparse(testdir)
        node = dom.find_first_by_tag("testsuite")
        tnode = node.find_first_by_tag("testcase")
        fnode = tnode.find_first_by_tag("failure")
        fnode.assert_attr(message="AssertionError: An error assert 0")

    def test_failure_escape(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.mark.parametrize('arg1', "<&'", ids="<&'")
            def test_func(arg1):
                print(arg1)
                assert 0
        """)
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=3, tests=3)

        for index, char in enumerate("<&'"):

            tnode = node.find_nth_by_tag("testcase", index)
            tnode.assert_attr(
                file="test_failure_escape.py",
                line="1",
                classname="test_failure_escape",
                name="test_func[%s]" % char)
            sysout = tnode.find_first_by_tag('system-out')
            text = sysout.text
            assert text == '%s\n' % char

    def test_junit_prefixing(self, testdir):
        testdir.makepyfile("""
            def test_func():
                assert 0
            class TestHello:
                def test_hello(self):
                    pass
        """)
        result, dom = runandparse(testdir, "--junitprefix=xyz")
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(failures=1, tests=2)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_junit_prefixing.py",
            line="0",
            classname="xyz.test_junit_prefixing",
            name="test_func")
        tnode = node.find_nth_by_tag("testcase", 1)
        tnode.assert_attr(
            file="test_junit_prefixing.py",
            line="3",
            classname="xyz.test_junit_prefixing."
            "TestHello",
            name="test_hello")

    def test_xfailure_function(self, testdir):
        testdir.makepyfile("""
            import pytest
            def test_xfail():
                pytest.xfail("42")
        """)
        result, dom = runandparse(testdir)
        assert not result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skips=1, tests=0)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_xfailure_function.py",
            line="1",
            classname="test_xfailure_function",
            name="test_xfail")
        fnode = tnode.find_first_by_tag("skipped")
        fnode.assert_attr(message="expected test failure")
        # assert "ValueError" in fnode.toxml()

    def test_xfailure_xpass(self, testdir):
        testdir.makepyfile("""
            import pytest
            @pytest.mark.xfail
            def test_xpass():
                pass
        """)
        result, dom = runandparse(testdir)
        # assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skips=1, tests=0)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_xfailure_xpass.py",
            line="1",
            classname="test_xfailure_xpass",
            name="test_xpass")
        fnode = tnode.find_first_by_tag("skipped")
        fnode.assert_attr(message="xfail-marked test passes unexpectedly")
        # assert "ValueError" in fnode.toxml()

    def test_collect_error(self, testdir):
        testdir.makepyfile("syntax error")
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=1, tests=0)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_collect_error.py",
            name="test_collect_error")
        assert tnode["line"] is None
        fnode = tnode.find_first_by_tag("error")
        fnode.assert_attr(message="collection failure")
        assert "SyntaxError" in fnode.toxml()

    def test_collect_skipped(self, testdir):
        testdir.makepyfile("import pytest; pytest.skip('xyz')")
        result, dom = runandparse(testdir)
        assert result.ret == EXIT_NOTESTSCOLLECTED
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(skips=1, tests=0)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(
            file="test_collect_skipped.py",
            name="test_collect_skipped")

        # py.test doesn't give us a line here.
        assert tnode["line"] is None

        fnode = tnode.find_first_by_tag("skipped")
        fnode.assert_attr(message="collection skipped")

    def test_unicode(self, testdir):
        value = 'hx\xc4\x85\xc4\x87\n'
        testdir.makepyfile("""
            # coding: latin1
            def test_hello():
                print (%r)
                assert 0
        """ % value)
        result, dom = runandparse(testdir)
        assert result.ret == 1
        tnode = dom.find_first_by_tag("testcase")
        fnode = tnode.find_first_by_tag("failure")
        if not sys.platform.startswith("java"):
            assert "hx" in fnode.toxml()

    def test_assertion_binchars(self, testdir):
        """this test did fail when the escaping wasnt strict"""
        testdir.makepyfile("""

            M1 = '\x01\x02\x03\x04'
            M2 = '\x01\x02\x03\x05'

            def test_str_compare():
                assert M1 == M2
            """)
        result, dom = runandparse(testdir)
        print(dom.toxml())

    def test_pass_captures_stdout(self, testdir):
        testdir.makepyfile("""
            def test_pass():
                print('hello-stdout')
        """)
        result, dom = runandparse(testdir)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        systemout = pnode.find_first_by_tag("system-out")
        assert "hello-stdout" in systemout.toxml()

    def test_pass_captures_stderr(self, testdir):
        testdir.makepyfile("""
            import sys
            def test_pass():
                sys.stderr.write('hello-stderr')
        """)
        result, dom = runandparse(testdir)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        systemout = pnode.find_first_by_tag("system-err")
        assert "hello-stderr" in systemout.toxml()

    def test_setup_error_captures_stdout(self, testdir):
        testdir.makepyfile("""
            def pytest_funcarg__arg(request):
                print('hello-stdout')
                raise ValueError()
            def test_function(arg):
                pass
        """)
        result, dom = runandparse(testdir)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        systemout = pnode.find_first_by_tag("system-out")
        assert "hello-stdout" in systemout.toxml()

    def test_setup_error_captures_stderr(self, testdir):
        testdir.makepyfile("""
            import sys
            def pytest_funcarg__arg(request):
                sys.stderr.write('hello-stderr')
                raise ValueError()
            def test_function(arg):
                pass
        """)
        result, dom = runandparse(testdir)
        node = dom.find_first_by_tag("testsuite")
        pnode = node.find_first_by_tag("testcase")
        systemout = pnode.find_first_by_tag("system-err")
        assert "hello-stderr" in systemout.toxml()


def test_mangle_test_address():
    from _pytest.junitxml import mangle_test_address
    address = '::'.join(
        ["a/my.py.thing.py", "Class", "()", "method", "[a-1-::]"])
    newnames = mangle_test_address(address)
    assert newnames == ["a.my.py.thing", "Class", "method", "[a-1-::]"]


def test_dont_configure_on_slaves(tmpdir):
    gotten = []

    class FakeConfig:
        def __init__(self):
            self.pluginmanager = self
            self.option = self

        junitprefix = None
        # XXX: shouldnt need tmpdir ?
        xmlpath = str(tmpdir.join('junix.xml'))
        register = gotten.append

    fake_config = FakeConfig()
    from _pytest import junitxml
    junitxml.pytest_configure(fake_config)
    assert len(gotten) == 1
    FakeConfig.slaveinput = None
    junitxml.pytest_configure(fake_config)
    assert len(gotten) == 1


class TestNonPython:
    def test_summing_simple(self, testdir):
        testdir.makeconftest("""
            import pytest
            def pytest_collect_file(path, parent):
                if path.ext == ".xyz":
                    return MyItem(path, parent)
            class MyItem(pytest.Item):
                def __init__(self, path, parent):
                    super(MyItem, self).__init__(path.basename, parent)
                    self.fspath = path
                def runtest(self):
                    raise ValueError(42)
                def repr_failure(self, excinfo):
                    return "custom item runtest failed"
        """)
        testdir.tmpdir.join("myfile.xyz").write("hello")
        result, dom = runandparse(testdir)
        assert result.ret
        node = dom.find_first_by_tag("testsuite")
        node.assert_attr(errors=0, failures=1, skips=0, tests=1)
        tnode = node.find_first_by_tag("testcase")
        tnode.assert_attr(name="myfile.xyz")
        fnode = tnode.find_first_by_tag("failure")
        fnode.assert_attr(message="custom item runtest failed")
        assert "custom item runtest failed" in fnode.toxml()


def test_nullbyte(testdir):
    # A null byte can not occur in XML (see section 2.2 of the spec)
    testdir.makepyfile("""
        import sys
        def test_print_nullbyte():
            sys.stdout.write('Here the null -->' + chr(0) + '<--')
            sys.stdout.write('In repr form -->' + repr(chr(0)) + '<--')
            assert False
    """)
    xmlf = testdir.tmpdir.join('junit.xml')
    testdir.runpytest('--junitxml=%s' % xmlf)
    text = xmlf.read()
    assert '\x00' not in text
    assert '#x00' in text


def test_nullbyte_replace(testdir):
    # Check if the null byte gets replaced
    testdir.makepyfile("""
        import sys
        def test_print_nullbyte():
            sys.stdout.write('Here the null -->' + chr(0) + '<--')
            sys.stdout.write('In repr form -->' + repr(chr(0)) + '<--')
            assert False
    """)
    xmlf = testdir.tmpdir.join('junit.xml')
    testdir.runpytest('--junitxml=%s' % xmlf)
    text = xmlf.read()
    assert '#x0' in text


def test_invalid_xml_escape():
    # Test some more invalid xml chars, the full range should be
    # tested really but let's just thest the edges of the ranges
    # intead.
    # XXX This only tests low unicode character points for now as
    #     there are some issues with the testing infrastructure for
    #     the higher ones.
    # XXX Testing 0xD (\r) is tricky as it overwrites the just written
    #     line in the output, so we skip it too.
    global unichr
    try:
        unichr(65)
    except NameError:
        unichr = chr
    invalid = (0x00, 0x1, 0xB, 0xC, 0xE, 0x19, 27,  # issue #126
               0xD800, 0xDFFF, 0xFFFE, 0x0FFFF)  # , 0x110000)
    valid = (0x9, 0xA, 0x20, )
    # 0xD, 0xD7FF, 0xE000, 0xFFFD, 0x10000, 0x10FFFF)

    from _pytest.junitxml import bin_xml_escape

    for i in invalid:
        got = bin_xml_escape(unichr(i)).uniobj
        if i <= 0xFF:
            expected = '#x%02X' % i
        else:
            expected = '#x%04X' % i
        assert got == expected
    for i in valid:
        assert chr(i) == bin_xml_escape(unichr(i)).uniobj


def test_logxml_path_expansion(tmpdir, monkeypatch):
    home_tilde = py.path.local(os.path.expanduser('~')).join('test.xml')

    xml_tilde = LogXML('~%stest.xml' % tmpdir.sep, None)
    assert xml_tilde.logfile == home_tilde

    # this is here for when $HOME is not set correct
    monkeypatch.setenv("HOME", tmpdir)
    home_var = os.path.normpath(os.path.expandvars('$HOME/test.xml'))

    xml_var = LogXML('$HOME%stest.xml' % tmpdir.sep, None)
    assert xml_var.logfile == home_var


def test_logxml_changingdir(testdir):
    testdir.makepyfile("""
        def test_func():
            import os
            os.chdir("a")
    """)
    testdir.tmpdir.mkdir("a")
    result = testdir.runpytest("--junitxml=a/x.xml")
    assert result.ret == 0
    assert testdir.tmpdir.join("a/x.xml").check()


def test_logxml_makedir(testdir):
    """--junitxml should automatically create directories for the xml file"""
    testdir.makepyfile("""
        def test_pass():
            pass
    """)
    result = testdir.runpytest("--junitxml=path/to/results.xml")
    assert result.ret == 0
    assert testdir.tmpdir.join("path/to/results.xml").check()


def test_escaped_parametrized_names_xml(testdir):
    testdir.makepyfile("""
        import pytest
        @pytest.mark.parametrize('char', ["\\x00"])
        def test_func(char):
            assert char
    """)
    result, dom = runandparse(testdir)
    assert result.ret == 0
    node = dom.find_first_by_tag("testcase")
    node.assert_attr(name="test_func[#x00]")


def test_double_colon_split_function_issue469(testdir):
    testdir.makepyfile("""
        import pytest
        @pytest.mark.parametrize('param', ["double::colon"])
        def test_func(param):
            pass
    """)
    result, dom = runandparse(testdir)
    assert result.ret == 0
    node = dom.find_first_by_tag("testcase")
    node.assert_attr(classname="test_double_colon_split_function_issue469")
    node.assert_attr(name='test_func[double::colon]')


def test_double_colon_split_method_issue469(testdir):
    testdir.makepyfile("""
        import pytest
        class TestClass:
            @pytest.mark.parametrize('param', ["double::colon"])
            def test_func(self, param):
                pass
    """)
    result, dom = runandparse(testdir)
    assert result.ret == 0
    node = dom.find_first_by_tag("testcase")
    node.assert_attr(
        classname="test_double_colon_split_method_issue469.TestClass")
    node.assert_attr(name='test_func[double::colon]')


def test_unicode_issue368(testdir):
    path = testdir.tmpdir.join("test.xml")
    log = LogXML(str(path), None)
    ustr = py.builtin._totext("ВНИ!", "utf-8")
    from _pytest.runner import BaseReport

    class Report(BaseReport):
        longrepr = ustr
        sections = []
        nodeid = "something"
        location = 'tests/filename.py', 42, 'TestClass.method'

    test_report = Report()

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
    test_report.wasxfail = ustr
    node_reporter.append_skipped(test_report)
    log.pytest_sessionfinish()


def test_record_property(testdir):
    testdir.makepyfile("""
        import pytest

        @pytest.fixture
        def other(record_xml_property):
            record_xml_property("bar", 1)
        def test_record(record_xml_property, other):
            record_xml_property("foo", "<1");
    """)
    result, dom = runandparse(testdir, '-rw')
    node = dom.find_first_by_tag("testsuite")
    tnode = node.find_first_by_tag("testcase")
    psnode = tnode.find_first_by_tag('properties')
    pnodes = psnode.find_by_tag('property')
    pnodes[0].assert_attr(name="bar", value="1")
    pnodes[1].assert_attr(name="foo", value="<1")
    result.stdout.fnmatch_lines('*C3*test_record_property.py*experimental*')


def test_record_property_same_name(testdir):
    testdir.makepyfile("""
        def test_record_with_same_name(record_xml_property):
            record_xml_property("foo", "bar")
            record_xml_property("foo", "baz")
    """)
    result, dom = runandparse(testdir, '-rw')
    node = dom.find_first_by_tag("testsuite")
    tnode = node.find_first_by_tag("testcase")
    psnode = tnode.find_first_by_tag('properties')
    pnodes = psnode.find_by_tag('property')
    pnodes[0].assert_attr(name="foo", value="bar")
    pnodes[1].assert_attr(name="foo", value="baz")


def test_random_report_log_xdist(testdir):
    """xdist calls pytest_runtest_logreport as they are executed by the slaves,
    with nodes from several nodes overlapping, so junitxml must cope with that
    to produce correct reports. #1064
    """
    pytest.importorskip('xdist')
    testdir.makepyfile("""
        import pytest, time
        @pytest.mark.parametrize('i', list(range(30)))
        def test_x(i):
            assert i != 22
    """)
    _, dom = runandparse(testdir, '-n2')
    suite_node = dom.find_first_by_tag("testsuite")
    failed = []
    for case_node in suite_node.find_by_tag("testcase"):
        if case_node.find_first_by_tag('failure'):
            failed.append(case_node['name'])

    assert failed == ['test_x[22]']


def test_runs_twice(testdir):
    f = testdir.makepyfile('''
        def test_pass():
            pass
    ''')

    result, dom = runandparse(testdir, f, f)
    assert 'INTERNALERROR' not in result.stdout.str()
    first, second = [x['classname'] for x in dom.find_by_tag("testcase")]
    assert first == second


@pytest.mark.xfail(reason='hangs', run=False)
def test_runs_twice_xdist(testdir):
    pytest.importorskip('xdist')
    f = testdir.makepyfile('''
        def test_pass():
            pass
    ''')

    result, dom = runandparse(
        testdir, f,
        '--dist', 'each', '--tx', '2*popen',)
    assert 'INTERNALERROR' not in result.stdout.str()
    first, second = [x['classname'] for x in dom.find_by_tag("testcase")]
    assert first == second


def test_fancy_items_regression(testdir):
    # issue 1259
    testdir.makeconftest("""
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
                    FunItem('a', self),
                    NoFunItem('a', self),
                    NoFunItem('b', self),
                ]

        def pytest_collect_file(path, parent):
            if path.check(ext='.py'):
                return FunCollector(path, parent)
    """)

    testdir.makepyfile('''
        def test_pass():
            pass
    ''')

    result, dom = runandparse(testdir)

    assert 'INTERNALERROR' not in result.stdout.str()

    items = sorted(
        '%(classname)s %(name)s %(file)s' % x

        for x in dom.find_by_tag("testcase"))
    import pprint
    pprint.pprint(items)
    assert items == [
        u'conftest a conftest.py',
        u'conftest a conftest.py',
        u'conftest b conftest.py',
        u'test_fancy_items_regression a test_fancy_items_regression.py',
        u'test_fancy_items_regression a test_fancy_items_regression.py',
        u'test_fancy_items_regression b test_fancy_items_regression.py',
        u'test_fancy_items_regression test_pass'
        u' test_fancy_items_regression.py',
    ]
