# -*- coding: utf-8 -*-
from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

import inspect
import sys
import textwrap

import pytest
from _pytest.compat import MODULE_NOT_FOUND_ERROR
from _pytest.doctest import _is_mocked
from _pytest.doctest import _patch_unwrap_mock_aware
from _pytest.doctest import DoctestItem
from _pytest.doctest import DoctestModule
from _pytest.doctest import DoctestTextfile


class TestDoctests(object):
    def test_collect_testtextfile(self, testdir):
        w = testdir.maketxtfile(whatever="")
        checkfile = testdir.maketxtfile(
            test_something="""
            alskdjalsdk
            >>> i = 5
            >>> i-1
            4
        """
        )

        for x in (testdir.tmpdir, checkfile):
            # print "checking that %s returns custom items" % (x,)
            items, reprec = testdir.inline_genitems(x)
            assert len(items) == 1
            assert isinstance(items[0], DoctestItem)
            assert isinstance(items[0].parent, DoctestTextfile)
        # Empty file has no items.
        items, reprec = testdir.inline_genitems(w)
        assert len(items) == 0

    def test_collect_module_empty(self, testdir):
        path = testdir.makepyfile(whatever="#")
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p, "--doctest-modules")
            assert len(items) == 0

    def test_collect_module_single_modulelevel_doctest(self, testdir):
        path = testdir.makepyfile(whatever='""">>> pass"""')
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p, "--doctest-modules")
            assert len(items) == 1
            assert isinstance(items[0], DoctestItem)
            assert isinstance(items[0].parent, DoctestModule)

    def test_collect_module_two_doctest_one_modulelevel(self, testdir):
        path = testdir.makepyfile(
            whatever="""
            '>>> x = None'
            def my_func():
                ">>> magic = 42 "
        """
        )
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p, "--doctest-modules")
            assert len(items) == 2
            assert isinstance(items[0], DoctestItem)
            assert isinstance(items[1], DoctestItem)
            assert isinstance(items[0].parent, DoctestModule)
            assert items[0].parent is items[1].parent

    def test_collect_module_two_doctest_no_modulelevel(self, testdir):
        path = testdir.makepyfile(
            whatever="""
            '# Empty'
            def my_func():
                ">>> magic = 42 "
            def unuseful():
                '''
                # This is a function
                # >>> # it doesn't have any doctest
                '''
            def another():
                '''
                # This is another function
                >>> import os # this one does have a doctest
                '''
        """
        )
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p, "--doctest-modules")
            assert len(items) == 2
            assert isinstance(items[0], DoctestItem)
            assert isinstance(items[1], DoctestItem)
            assert isinstance(items[0].parent, DoctestModule)
            assert items[0].parent is items[1].parent

    def test_simple_doctestfile(self, testdir):
        p = testdir.maketxtfile(
            test_doc="""
            >>> x = 1
            >>> x == 1
            False
        """
        )
        reprec = testdir.inline_run(p)
        reprec.assertoutcome(failed=1)

    def test_new_pattern(self, testdir):
        p = testdir.maketxtfile(
            xdoc="""
            >>> x = 1
            >>> x == 1
            False
        """
        )
        reprec = testdir.inline_run(p, "--doctest-glob=x*.txt")
        reprec.assertoutcome(failed=1)

    def test_multiple_patterns(self, testdir):
        """Test support for multiple --doctest-glob arguments (#1255).
        """
        testdir.maketxtfile(
            xdoc="""
            >>> 1
            1
        """
        )
        testdir.makefile(
            ".foo",
            test="""
            >>> 1
            1
        """,
        )
        testdir.maketxtfile(
            test_normal="""
            >>> 1
            1
        """
        )
        expected = {"xdoc.txt", "test.foo", "test_normal.txt"}
        assert {x.basename for x in testdir.tmpdir.listdir()} == expected
        args = ["--doctest-glob=xdoc*.txt", "--doctest-glob=*.foo"]
        result = testdir.runpytest(*args)
        result.stdout.fnmatch_lines(["*test.foo *", "*xdoc.txt *", "*2 passed*"])
        result = testdir.runpytest()
        result.stdout.fnmatch_lines(["*test_normal.txt *", "*1 passed*"])

    @pytest.mark.parametrize(
        "   test_string,    encoding",
        [(u"foo", "ascii"), (u"öäü", "latin1"), (u"öäü", "utf-8")],
    )
    def test_encoding(self, testdir, test_string, encoding):
        """Test support for doctest_encoding ini option.
        """
        testdir.makeini(
            """
            [pytest]
            doctest_encoding={}
        """.format(
                encoding
            )
        )
        doctest = u"""
            >>> u"{}"
            {}
        """.format(
            test_string, repr(test_string)
        )
        testdir._makefile(".txt", [doctest], {}, encoding=encoding)

        result = testdir.runpytest()

        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_doctest_unexpected_exception(self, testdir):
        testdir.maketxtfile(
            """
            >>> i = 0
            >>> 0 / i
            2
        """
        )
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines(
            [
                "*unexpected_exception*",
                "*>>> i = 0*",
                "*>>> 0 / i*",
                "*UNEXPECTED*ZeroDivision*",
            ]
        )

    def test_doctest_skip(self, testdir):
        testdir.maketxtfile(
            """
            >>> 1
            1
            >>> import pytest
            >>> pytest.skip("")
        """
        )
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines(["*1 skipped*"])

    def test_docstring_partial_context_around_error(self, testdir):
        """Test that we show some context before the actual line of a failing
        doctest.
        """
        testdir.makepyfile(
            '''
            def foo():
                """
                text-line-1
                text-line-2
                text-line-3
                text-line-4
                text-line-5
                text-line-6
                text-line-7
                text-line-8
                text-line-9
                text-line-10
                text-line-11
                >>> 1 + 1
                3

                text-line-after
                """
        '''
        )
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines(
            [
                "*docstring_partial_context_around_error*",
                "005*text-line-3",
                "006*text-line-4",
                "013*text-line-11",
                "014*>>> 1 + 1",
                "Expected:",
                "    3",
                "Got:",
                "    2",
            ]
        )
        # lines below should be trimmed out
        assert "text-line-2" not in result.stdout.str()
        assert "text-line-after" not in result.stdout.str()

    def test_docstring_full_context_around_error(self, testdir):
        """Test that we show the whole context before the actual line of a failing
        doctest, provided that the context is up to 10 lines long.
        """
        testdir.makepyfile(
            '''
            def foo():
                """
                text-line-1
                text-line-2

                >>> 1 + 1
                3
                """
        '''
        )
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines(
            [
                "*docstring_full_context_around_error*",
                "003*text-line-1",
                "004*text-line-2",
                "006*>>> 1 + 1",
                "Expected:",
                "    3",
                "Got:",
                "    2",
            ]
        )

    def test_doctest_linedata_missing(self, testdir):
        testdir.tmpdir.join("hello.py").write(
            textwrap.dedent(
                """\
                class Fun(object):
                    @property
                    def test(self):
                        '''
                        >>> a = 1
                        >>> 1/0
                        '''
                """
            )
        )
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines(
            [
                "*hello*",
                "*EXAMPLE LOCATION UNKNOWN, not showing all tests of that example*",
                "*1/0*",
                "*UNEXPECTED*ZeroDivision*",
                "*1 failed*",
            ]
        )

    def test_doctest_unex_importerror_only_txt(self, testdir):
        testdir.maketxtfile(
            """
            >>> import asdalsdkjaslkdjasd
            >>>
        """
        )
        result = testdir.runpytest()
        # doctest is never executed because of error during hello.py collection
        result.stdout.fnmatch_lines(
            [
                "*>>> import asdals*",
                "*UNEXPECTED*{e}*".format(e=MODULE_NOT_FOUND_ERROR),
                "{e}: No module named *asdal*".format(e=MODULE_NOT_FOUND_ERROR),
            ]
        )

    def test_doctest_unex_importerror_with_module(self, testdir):
        testdir.tmpdir.join("hello.py").write(
            textwrap.dedent(
                """\
                import asdalsdkjaslkdjasd
                """
            )
        )
        testdir.maketxtfile(
            """
            >>> import hello
            >>>
        """
        )
        result = testdir.runpytest("--doctest-modules")
        # doctest is never executed because of error during hello.py collection
        result.stdout.fnmatch_lines(
            [
                "*ERROR collecting hello.py*",
                "*{e}: No module named *asdals*".format(e=MODULE_NOT_FOUND_ERROR),
                "*Interrupted: 1 errors during collection*",
            ]
        )

    def test_doctestmodule(self, testdir):
        p = testdir.makepyfile(
            """
            '''
                >>> x = 1
                >>> x == 1
                False

            '''
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(failed=1)

    def test_doctestmodule_external_and_issue116(self, testdir):
        p = testdir.mkpydir("hello")
        p.join("__init__.py").write(
            textwrap.dedent(
                """\
                def somefunc():
                    '''
                        >>> i = 0
                        >>> i + 1
                        2
                    '''
                """
            )
        )
        result = testdir.runpytest(p, "--doctest-modules")
        result.stdout.fnmatch_lines(
            [
                "003 *>>> i = 0",
                "004 *>>> i + 1",
                "*Expected:",
                "*    2",
                "*Got:",
                "*    1",
                "*:4: DocTestFailure",
            ]
        )

    def test_txtfile_failing(self, testdir):
        p = testdir.maketxtfile(
            """
            >>> i = 0
            >>> i + 1
            2
        """
        )
        result = testdir.runpytest(p, "-s")
        result.stdout.fnmatch_lines(
            [
                "001 >>> i = 0",
                "002 >>> i + 1",
                "Expected:",
                "    2",
                "Got:",
                "    1",
                "*test_txtfile_failing.txt:2: DocTestFailure",
            ]
        )

    def test_txtfile_with_fixtures(self, testdir):
        p = testdir.maketxtfile(
            """
            >>> dir = getfixture('tmpdir')
            >>> type(dir).__name__
            'LocalPath'
        """
        )
        reprec = testdir.inline_run(p)
        reprec.assertoutcome(passed=1)

    def test_txtfile_with_usefixtures_in_ini(self, testdir):
        testdir.makeini(
            """
            [pytest]
            usefixtures = myfixture
        """
        )
        testdir.makeconftest(
            """
            import pytest
            @pytest.fixture
            def myfixture(monkeypatch):
                monkeypatch.setenv("HELLO", "WORLD")
        """
        )

        p = testdir.maketxtfile(
            """
            >>> import os
            >>> os.environ["HELLO"]
            'WORLD'
        """
        )
        reprec = testdir.inline_run(p)
        reprec.assertoutcome(passed=1)

    def test_doctestmodule_with_fixtures(self, testdir):
        p = testdir.makepyfile(
            """
            '''
                >>> dir = getfixture('tmpdir')
                >>> type(dir).__name__
                'LocalPath'
            '''
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(passed=1)

    def test_doctestmodule_three_tests(self, testdir):
        p = testdir.makepyfile(
            """
            '''
            >>> dir = getfixture('tmpdir')
            >>> type(dir).__name__
            'LocalPath'
            '''
            def my_func():
                '''
                >>> magic = 42
                >>> magic - 42
                0
                '''
            def unuseful():
                pass
            def another():
                '''
                >>> import os
                >>> os is os
                True
                '''
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(passed=3)

    def test_doctestmodule_two_tests_one_fail(self, testdir):
        p = testdir.makepyfile(
            """
            class MyClass(object):
                def bad_meth(self):
                    '''
                    >>> magic = 42
                    >>> magic
                    0
                    '''
                def nice_meth(self):
                    '''
                    >>> magic = 42
                    >>> magic - 42
                    0
                    '''
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(failed=1, passed=1)

    def test_ignored_whitespace(self, testdir):
        testdir.makeini(
            """
            [pytest]
            doctest_optionflags = ELLIPSIS NORMALIZE_WHITESPACE
        """
        )
        p = testdir.makepyfile(
            """
            class MyClass(object):
                '''
                >>> a = "foo    "
                >>> print(a)
                foo
                '''
                pass
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(passed=1)

    def test_non_ignored_whitespace(self, testdir):
        testdir.makeini(
            """
            [pytest]
            doctest_optionflags = ELLIPSIS
        """
        )
        p = testdir.makepyfile(
            """
            class MyClass(object):
                '''
                >>> a = "foo    "
                >>> print(a)
                foo
                '''
                pass
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(failed=1, passed=0)

    def test_ignored_whitespace_glob(self, testdir):
        testdir.makeini(
            """
            [pytest]
            doctest_optionflags = ELLIPSIS NORMALIZE_WHITESPACE
        """
        )
        p = testdir.maketxtfile(
            xdoc="""
            >>> a = "foo    "
            >>> print(a)
            foo
        """
        )
        reprec = testdir.inline_run(p, "--doctest-glob=x*.txt")
        reprec.assertoutcome(passed=1)

    def test_non_ignored_whitespace_glob(self, testdir):
        testdir.makeini(
            """
            [pytest]
            doctest_optionflags = ELLIPSIS
        """
        )
        p = testdir.maketxtfile(
            xdoc="""
            >>> a = "foo    "
            >>> print(a)
            foo
        """
        )
        reprec = testdir.inline_run(p, "--doctest-glob=x*.txt")
        reprec.assertoutcome(failed=1, passed=0)

    def test_contains_unicode(self, testdir):
        """Fix internal error with docstrings containing non-ascii characters.
        """
        testdir.makepyfile(
            u'''
            # -*- coding: utf-8 -*-
            def foo():
                """
                >>> name = 'с' # not letter 'c' but instead Cyrillic 's'.
                'anything'
                """
        '''
        )
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines(["Got nothing", "* 1 failed in*"])

    def test_ignore_import_errors_on_doctest(self, testdir):
        p = testdir.makepyfile(
            """
            import asdf

            def add_one(x):
                '''
                >>> add_one(1)
                2
                '''
                return x + 1
        """
        )

        reprec = testdir.inline_run(
            p, "--doctest-modules", "--doctest-ignore-import-errors"
        )
        reprec.assertoutcome(skipped=1, failed=1, passed=0)

    def test_junit_report_for_doctest(self, testdir):
        """
        #713: Fix --junit-xml option when used with --doctest-modules.
        """
        p = testdir.makepyfile(
            """
            def foo():
                '''
                >>> 1 + 1
                3
                '''
                pass
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules", "--junit-xml=junit.xml")
        reprec.assertoutcome(failed=1)

    def test_unicode_doctest(self, testdir):
        """
        Test case for issue 2434: DecodeError on Python 2 when doctest contains non-ascii
        characters.
        """
        p = testdir.maketxtfile(
            test_unicode_doctest="""
            .. doctest::

                >>> print(
                ...    "Hi\\n\\nByé")
                Hi
                ...
                Byé
                >>> 1/0  # Byé
                1
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(
            ["*UNEXPECTED EXCEPTION: ZeroDivisionError*", "*1 failed*"]
        )

    def test_unicode_doctest_module(self, testdir):
        """
        Test case for issue 2434: DecodeError on Python 2 when doctest docstring
        contains non-ascii characters.
        """
        p = testdir.makepyfile(
            test_unicode_doctest_module="""
            # -*- coding: utf-8 -*-
            from __future__ import unicode_literals

            def fix_bad_unicode(text):
                '''
                    >>> print(fix_bad_unicode('Ãºnico'))
                    único
                '''
                return "único"
        """
        )
        result = testdir.runpytest(p, "--doctest-modules")
        result.stdout.fnmatch_lines(["* 1 passed *"])

    def test_print_unicode_value(self, testdir):
        """
        Test case for issue 3583: Printing Unicode in doctest under Python 2.7
        doesn't work
        """
        p = testdir.maketxtfile(
            test_print_unicode_value=r"""
            Here is a doctest::

                >>> print(u'\xE5\xE9\xEE\xF8\xFC')
                åéîøü
        """
        )
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines(["* 1 passed *"])

    def test_reportinfo(self, testdir):
        """
        Test case to make sure that DoctestItem.reportinfo() returns lineno.
        """
        p = testdir.makepyfile(
            test_reportinfo="""
            def foo(x):
                '''
                    >>> foo('a')
                    'b'
                '''
                return 'c'
        """
        )
        items, reprec = testdir.inline_genitems(p, "--doctest-modules")
        reportinfo = items[0].reportinfo()
        assert reportinfo[1] == 1

    def test_valid_setup_py(self, testdir):
        """
        Test to make sure that pytest ignores valid setup.py files when ran
        with --doctest-modules
        """
        p = testdir.makepyfile(
            setup="""
            from setuptools import setup, find_packages
            setup(name='sample',
                  version='0.0',
                  description='description',
                  packages=find_packages()
            )
        """
        )
        result = testdir.runpytest(p, "--doctest-modules")
        result.stdout.fnmatch_lines(["*collected 0 items*"])

    def test_invalid_setup_py(self, testdir):
        """
        Test to make sure that pytest reads setup.py files that are not used
        for python packages when ran with --doctest-modules
        """
        p = testdir.makepyfile(
            setup="""
            def test_foo():
                return 'bar'
        """
        )
        result = testdir.runpytest(p, "--doctest-modules")
        result.stdout.fnmatch_lines(["*collected 1 item*"])


class TestLiterals(object):
    @pytest.mark.parametrize("config_mode", ["ini", "comment"])
    def test_allow_unicode(self, testdir, config_mode):
        """Test that doctests which output unicode work in all python versions
        tested by pytest when the ALLOW_UNICODE option is used (either in
        the ini file or by an inline comment).
        """
        if config_mode == "ini":
            testdir.makeini(
                """
            [pytest]
            doctest_optionflags = ALLOW_UNICODE
            """
            )
            comment = ""
        else:
            comment = "#doctest: +ALLOW_UNICODE"

        testdir.maketxtfile(
            test_doc="""
            >>> b'12'.decode('ascii') {comment}
            '12'
        """.format(
                comment=comment
            )
        )
        testdir.makepyfile(
            foo="""
            def foo():
              '''
              >>> b'12'.decode('ascii') {comment}
              '12'
              '''
        """.format(
                comment=comment
            )
        )
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(passed=2)

    @pytest.mark.parametrize("config_mode", ["ini", "comment"])
    def test_allow_bytes(self, testdir, config_mode):
        """Test that doctests which output bytes work in all python versions
        tested by pytest when the ALLOW_BYTES option is used (either in
        the ini file or by an inline comment)(#1287).
        """
        if config_mode == "ini":
            testdir.makeini(
                """
            [pytest]
            doctest_optionflags = ALLOW_BYTES
            """
            )
            comment = ""
        else:
            comment = "#doctest: +ALLOW_BYTES"

        testdir.maketxtfile(
            test_doc="""
            >>> b'foo'  {comment}
            'foo'
        """.format(
                comment=comment
            )
        )
        testdir.makepyfile(
            foo="""
            def foo():
              '''
              >>> b'foo'  {comment}
              'foo'
              '''
        """.format(
                comment=comment
            )
        )
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(passed=2)

    def test_unicode_string(self, testdir):
        """Test that doctests which output unicode fail in Python 2 when
        the ALLOW_UNICODE option is not used. The same test should pass
        in Python 3.
        """
        testdir.maketxtfile(
            test_doc="""
            >>> b'12'.decode('ascii')
            '12'
        """
        )
        reprec = testdir.inline_run()
        passed = int(sys.version_info[0] >= 3)
        reprec.assertoutcome(passed=passed, failed=int(not passed))

    def test_bytes_literal(self, testdir):
        """Test that doctests which output bytes fail in Python 3 when
        the ALLOW_BYTES option is not used. The same test should pass
        in Python 2 (#1287).
        """
        testdir.maketxtfile(
            test_doc="""
            >>> b'foo'
            'foo'
        """
        )
        reprec = testdir.inline_run()
        passed = int(sys.version_info[0] == 2)
        reprec.assertoutcome(passed=passed, failed=int(not passed))


class TestDoctestSkips(object):
    """
    If all examples in a doctest are skipped due to the SKIP option, then
    the tests should be SKIPPED rather than PASSED. (#957)
    """

    @pytest.fixture(params=["text", "module"])
    def makedoctest(self, testdir, request):
        def makeit(doctest):
            mode = request.param
            if mode == "text":
                testdir.maketxtfile(doctest)
            else:
                assert mode == "module"
                testdir.makepyfile('"""\n%s"""' % doctest)

        return makeit

    def test_one_skipped(self, testdir, makedoctest):
        makedoctest(
            """
            >>> 1 + 1  # doctest: +SKIP
            2
            >>> 2 + 2
            4
        """
        )
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(passed=1)

    def test_one_skipped_failed(self, testdir, makedoctest):
        makedoctest(
            """
            >>> 1 + 1  # doctest: +SKIP
            2
            >>> 2 + 2
            200
        """
        )
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(failed=1)

    def test_all_skipped(self, testdir, makedoctest):
        makedoctest(
            """
            >>> 1 + 1  # doctest: +SKIP
            2
            >>> 2 + 2  # doctest: +SKIP
            200
        """
        )
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(skipped=1)

    def test_vacuous_all_skipped(self, testdir, makedoctest):
        makedoctest("")
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(passed=0, skipped=0)

    def test_continue_on_failure(self, testdir):
        testdir.maketxtfile(
            test_something="""
            >>> i = 5
            >>> def foo():
            ...     raise ValueError('error1')
            >>> foo()
            >>> i
            >>> i + 2
            7
            >>> i + 1
        """
        )
        result = testdir.runpytest("--doctest-modules", "--doctest-continue-on-failure")
        result.assert_outcomes(passed=0, failed=1)
        # The lines that contains the failure are 4, 5, and 8.  The first one
        # is a stack trace and the other two are mismatches.
        result.stdout.fnmatch_lines(
            ["*4: UnexpectedException*", "*5: DocTestFailure*", "*8: DocTestFailure*"]
        )


class TestDoctestAutoUseFixtures(object):

    SCOPES = ["module", "session", "class", "function"]

    def test_doctest_module_session_fixture(self, testdir):
        """Test that session fixtures are initialized for doctest modules (#768)
        """
        # session fixture which changes some global data, which will
        # be accessed by doctests in a module
        testdir.makeconftest(
            """
            import pytest
            import sys

            @pytest.yield_fixture(autouse=True, scope='session')
            def myfixture():
                assert not hasattr(sys, 'pytest_session_data')
                sys.pytest_session_data = 1
                yield
                del sys.pytest_session_data
        """
        )
        testdir.makepyfile(
            foo="""
            import sys

            def foo():
              '''
              >>> assert sys.pytest_session_data == 1
              '''

            def bar():
              '''
              >>> assert sys.pytest_session_data == 1
              '''
        """
        )
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines(["*2 passed*"])

    @pytest.mark.parametrize("scope", SCOPES)
    @pytest.mark.parametrize("enable_doctest", [True, False])
    def test_fixture_scopes(self, testdir, scope, enable_doctest):
        """Test that auto-use fixtures work properly with doctest modules.
        See #1057 and #1100.
        """
        testdir.makeconftest(
            """
            import pytest

            @pytest.fixture(autouse=True, scope="{scope}")
            def auto(request):
                return 99
        """.format(
                scope=scope
            )
        )
        testdir.makepyfile(
            test_1='''
            def test_foo():
                """
                >>> getfixture('auto') + 1
                100
                """
            def test_bar():
                assert 1
        '''
        )
        params = ("--doctest-modules",) if enable_doctest else ()
        passes = 3 if enable_doctest else 2
        result = testdir.runpytest(*params)
        result.stdout.fnmatch_lines(["*=== %d passed in *" % passes])

    @pytest.mark.parametrize("scope", SCOPES)
    @pytest.mark.parametrize("autouse", [True, False])
    @pytest.mark.parametrize("use_fixture_in_doctest", [True, False])
    def test_fixture_module_doctest_scopes(
        self, testdir, scope, autouse, use_fixture_in_doctest
    ):
        """Test that auto-use fixtures work properly with doctest files.
        See #1057 and #1100.
        """
        testdir.makeconftest(
            """
            import pytest

            @pytest.fixture(autouse={autouse}, scope="{scope}")
            def auto(request):
                return 99
        """.format(
                scope=scope, autouse=autouse
            )
        )
        if use_fixture_in_doctest:
            testdir.maketxtfile(
                test_doc="""
                >>> getfixture('auto')
                99
            """
            )
        else:
            testdir.maketxtfile(
                test_doc="""
                >>> 1 + 1
                2
            """
            )
        result = testdir.runpytest("--doctest-modules")
        assert "FAILURES" not in str(result.stdout.str())
        result.stdout.fnmatch_lines(["*=== 1 passed in *"])

    @pytest.mark.parametrize("scope", SCOPES)
    def test_auto_use_request_attributes(self, testdir, scope):
        """Check that all attributes of a request in an autouse fixture
        behave as expected when requested for a doctest item.
        """
        testdir.makeconftest(
            """
            import pytest

            @pytest.fixture(autouse=True, scope="{scope}")
            def auto(request):
                if "{scope}" == 'module':
                    assert request.module is None
                if "{scope}" == 'class':
                    assert request.cls is None
                if "{scope}" == 'function':
                    assert request.function is None
                return 99
        """.format(
                scope=scope
            )
        )
        testdir.maketxtfile(
            test_doc="""
            >>> 1 + 1
            2
        """
        )
        result = testdir.runpytest("--doctest-modules")
        assert "FAILURES" not in str(result.stdout.str())
        result.stdout.fnmatch_lines(["*=== 1 passed in *"])


class TestDoctestNamespaceFixture(object):

    SCOPES = ["module", "session", "class", "function"]

    @pytest.mark.parametrize("scope", SCOPES)
    def test_namespace_doctestfile(self, testdir, scope):
        """
        Check that inserting something into the namespace works in a
        simple text file doctest
        """
        testdir.makeconftest(
            """
            import pytest
            import contextlib

            @pytest.fixture(autouse=True, scope="{scope}")
            def add_contextlib(doctest_namespace):
                doctest_namespace['cl'] = contextlib
        """.format(
                scope=scope
            )
        )
        p = testdir.maketxtfile(
            """
            >>> print(cl.__name__)
            contextlib
        """
        )
        reprec = testdir.inline_run(p)
        reprec.assertoutcome(passed=1)

    @pytest.mark.parametrize("scope", SCOPES)
    def test_namespace_pyfile(self, testdir, scope):
        """
        Check that inserting something into the namespace works in a
        simple Python file docstring doctest
        """
        testdir.makeconftest(
            """
            import pytest
            import contextlib

            @pytest.fixture(autouse=True, scope="{scope}")
            def add_contextlib(doctest_namespace):
                doctest_namespace['cl'] = contextlib
        """.format(
                scope=scope
            )
        )
        p = testdir.makepyfile(
            """
            def foo():
                '''
                >>> print(cl.__name__)
                contextlib
                '''
        """
        )
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(passed=1)


class TestDoctestReportingOption(object):
    def _run_doctest_report(self, testdir, format):
        testdir.makepyfile(
            """
            def foo():
                '''
                >>> foo()
                   a  b
                0  1  4
                1  2  4
                2  3  6
                '''
                print('   a  b\\n'
                      '0  1  4\\n'
                      '1  2  5\\n'
                      '2  3  6')
            """
        )
        return testdir.runpytest("--doctest-modules", "--doctest-report", format)

    @pytest.mark.parametrize("format", ["udiff", "UDIFF", "uDiFf"])
    def test_doctest_report_udiff(self, testdir, format):
        result = self._run_doctest_report(testdir, format)
        result.stdout.fnmatch_lines(
            ["     0  1  4", "    -1  2  4", "    +1  2  5", "     2  3  6"]
        )

    def test_doctest_report_cdiff(self, testdir):
        result = self._run_doctest_report(testdir, "cdiff")
        result.stdout.fnmatch_lines(
            [
                "         a  b",
                "      0  1  4",
                "    ! 1  2  4",
                "      2  3  6",
                "    --- 1,4 ----",
                "         a  b",
                "      0  1  4",
                "    ! 1  2  5",
                "      2  3  6",
            ]
        )

    def test_doctest_report_ndiff(self, testdir):
        result = self._run_doctest_report(testdir, "ndiff")
        result.stdout.fnmatch_lines(
            [
                "         a  b",
                "      0  1  4",
                "    - 1  2  4",
                "    ?       ^",
                "    + 1  2  5",
                "    ?       ^",
                "      2  3  6",
            ]
        )

    @pytest.mark.parametrize("format", ["none", "only_first_failure"])
    def test_doctest_report_none_or_only_first_failure(self, testdir, format):
        result = self._run_doctest_report(testdir, format)
        result.stdout.fnmatch_lines(
            [
                "Expected:",
                "       a  b",
                "    0  1  4",
                "    1  2  4",
                "    2  3  6",
                "Got:",
                "       a  b",
                "    0  1  4",
                "    1  2  5",
                "    2  3  6",
            ]
        )

    def test_doctest_report_invalid(self, testdir):
        result = self._run_doctest_report(testdir, "obviously_invalid_format")
        result.stderr.fnmatch_lines(
            [
                "*error: argument --doctest-report: invalid choice: 'obviously_invalid_format' (choose from*"
            ]
        )


@pytest.mark.parametrize("mock_module", ["mock", "unittest.mock"])
def test_doctest_mock_objects_dont_recurse_missbehaved(mock_module, testdir):
    pytest.importorskip(mock_module)
    testdir.makepyfile(
        """
        from {mock_module} import call
        class Example(object):
            '''
            >>> 1 + 1
            2
            '''
        """.format(
            mock_module=mock_module
        )
    )
    result = testdir.runpytest("--doctest-modules")
    result.stdout.fnmatch_lines(["* 1 passed *"])


class Broken:
    def __getattr__(self, _):
        raise KeyError("This should be an AttributeError")


@pytest.mark.skipif(not hasattr(inspect, "unwrap"), reason="nothing to patch")
@pytest.mark.parametrize(  # pragma: no branch (lambdas are not called)
    "stop", [None, _is_mocked, lambda f: None, lambda f: False, lambda f: True]
)
def test_warning_on_unwrap_of_broken_object(stop):
    bad_instance = Broken()
    assert inspect.unwrap.__module__ == "inspect"
    with _patch_unwrap_mock_aware():
        assert inspect.unwrap.__module__ != "inspect"
        with pytest.warns(
            pytest.PytestWarning, match="^Got KeyError.* when unwrapping"
        ):
            with pytest.raises(KeyError):
                inspect.unwrap(bad_instance, stop=stop)
    assert inspect.unwrap.__module__ == "inspect"
