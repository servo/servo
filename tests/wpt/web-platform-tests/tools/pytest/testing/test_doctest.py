# encoding: utf-8
import sys
import _pytest._code
from _pytest.doctest import DoctestItem, DoctestModule, DoctestTextfile
import pytest

class TestDoctests:

    def test_collect_testtextfile(self, testdir):
        w = testdir.maketxtfile(whatever="")
        checkfile = testdir.maketxtfile(test_something="""
            alskdjalsdk
            >>> i = 5
            >>> i-1
            4
        """)
        for x in (testdir.tmpdir, checkfile):
            #print "checking that %s returns custom items" % (x,)
            items, reprec = testdir.inline_genitems(x)
            assert len(items) == 1
            assert isinstance(items[0], DoctestTextfile)
        items, reprec = testdir.inline_genitems(w)
        assert len(items) == 1

    def test_collect_module_empty(self, testdir):
        path = testdir.makepyfile(whatever="#")
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p,
                '--doctest-modules')
            assert len(items) == 0

    def test_collect_module_single_modulelevel_doctest(self, testdir):
        path = testdir.makepyfile(whatever='""">>> pass"""')
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p,
                '--doctest-modules')
            assert len(items) == 1
            assert isinstance(items[0], DoctestItem)
            assert isinstance(items[0].parent, DoctestModule)

    def test_collect_module_two_doctest_one_modulelevel(self, testdir):
        path = testdir.makepyfile(whatever="""
            '>>> x = None'
            def my_func():
                ">>> magic = 42 "
        """)
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p,
                '--doctest-modules')
            assert len(items) == 2
            assert isinstance(items[0], DoctestItem)
            assert isinstance(items[1], DoctestItem)
            assert isinstance(items[0].parent, DoctestModule)
            assert items[0].parent is items[1].parent

    def test_collect_module_two_doctest_no_modulelevel(self, testdir):
        path = testdir.makepyfile(whatever="""
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
        """)
        for p in (path, testdir.tmpdir):
            items, reprec = testdir.inline_genitems(p,
                '--doctest-modules')
            assert len(items) == 2
            assert isinstance(items[0], DoctestItem)
            assert isinstance(items[1], DoctestItem)
            assert isinstance(items[0].parent, DoctestModule)
            assert items[0].parent is items[1].parent

    def test_simple_doctestfile(self, testdir):
        p = testdir.maketxtfile(test_doc="""
            >>> x = 1
            >>> x == 1
            False
        """)
        reprec = testdir.inline_run(p, )
        reprec.assertoutcome(failed=1)

    def test_new_pattern(self, testdir):
        p = testdir.maketxtfile(xdoc="""
            >>> x = 1
            >>> x == 1
            False
        """)
        reprec = testdir.inline_run(p, "--doctest-glob=x*.txt")
        reprec.assertoutcome(failed=1)

    def test_multiple_patterns(self, testdir):
        """Test support for multiple --doctest-glob arguments (#1255).
        """
        testdir.maketxtfile(xdoc="""
            >>> 1
            1
        """)
        testdir.makefile('.foo', test="""
            >>> 1
            1
        """)
        testdir.maketxtfile(test_normal="""
            >>> 1
            1
        """)
        expected = set(['xdoc.txt', 'test.foo', 'test_normal.txt'])
        assert set(x.basename for x in testdir.tmpdir.listdir()) == expected
        args = ["--doctest-glob=xdoc*.txt", "--doctest-glob=*.foo"]
        result = testdir.runpytest(*args)
        result.stdout.fnmatch_lines([
            '*test.foo *',
            '*xdoc.txt *',
            '*2 passed*',
        ])
        result = testdir.runpytest()
        result.stdout.fnmatch_lines([
            '*test_normal.txt *',
            '*1 passed*',
        ])

    def test_doctest_unexpected_exception(self, testdir):
        testdir.maketxtfile("""
            >>> i = 0
            >>> 0 / i
            2
        """)
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines([
            "*unexpected_exception*",
            "*>>> i = 0*",
            "*>>> 0 / i*",
            "*UNEXPECTED*ZeroDivision*",
        ])

    def test_docstring_context_around_error(self, testdir):
        """Test that we show some context before the actual line of a failing
        doctest.
        """
        testdir.makepyfile('''
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
        ''')
        result = testdir.runpytest('--doctest-modules')
        result.stdout.fnmatch_lines([
            '*docstring_context_around_error*',
            '005*text-line-3',
            '006*text-line-4',
            '013*text-line-11',
            '014*>>> 1 + 1',
            'Expected:',
            '    3',
            'Got:',
            '    2',
        ])
        # lines below should be trimmed out
        assert 'text-line-2' not in result.stdout.str()
        assert 'text-line-after' not in result.stdout.str()

    def test_doctest_linedata_missing(self, testdir):
        testdir.tmpdir.join('hello.py').write(_pytest._code.Source("""
            class Fun(object):
                @property
                def test(self):
                    '''
                    >>> a = 1
                    >>> 1/0
                    '''
            """))
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines([
            "*hello*",
            "*EXAMPLE LOCATION UNKNOWN, not showing all tests of that example*",
            "*1/0*",
            "*UNEXPECTED*ZeroDivision*",
            "*1 failed*",
        ])


    def test_doctest_unex_importerror(self, testdir):
        testdir.tmpdir.join("hello.py").write(_pytest._code.Source("""
            import asdalsdkjaslkdjasd
        """))
        testdir.maketxtfile("""
            >>> import hello
            >>>
        """)
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines([
            "*>>> import hello",
            "*UNEXPECTED*ImportError*",
            "*import asdals*",
        ])

    def test_doctestmodule(self, testdir):
        p = testdir.makepyfile("""
            '''
                >>> x = 1
                >>> x == 1
                False

            '''
        """)
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(failed=1)

    def test_doctestmodule_external_and_issue116(self, testdir):
        p = testdir.mkpydir("hello")
        p.join("__init__.py").write(_pytest._code.Source("""
            def somefunc():
                '''
                    >>> i = 0
                    >>> i + 1
                    2
                '''
        """))
        result = testdir.runpytest(p, "--doctest-modules")
        result.stdout.fnmatch_lines([
            '004 *>>> i = 0',
            '005 *>>> i + 1',
            '*Expected:',
            "*    2",
            "*Got:",
            "*    1",
            "*:5: DocTestFailure"
        ])


    def test_txtfile_failing(self, testdir):
        p = testdir.maketxtfile("""
            >>> i = 0
            >>> i + 1
            2
        """)
        result = testdir.runpytest(p, "-s")
        result.stdout.fnmatch_lines([
            '001 >>> i = 0',
            '002 >>> i + 1',
            'Expected:',
            "    2",
            "Got:",
            "    1",
            "*test_txtfile_failing.txt:2: DocTestFailure"
        ])

    def test_txtfile_with_fixtures(self, testdir):
        p = testdir.maketxtfile("""
            >>> dir = getfixture('tmpdir')
            >>> type(dir).__name__
            'LocalPath'
        """)
        reprec = testdir.inline_run(p, )
        reprec.assertoutcome(passed=1)

    def test_txtfile_with_usefixtures_in_ini(self, testdir):
        testdir.makeini("""
            [pytest]
            usefixtures = myfixture
        """)
        testdir.makeconftest("""
            import pytest
            @pytest.fixture
            def myfixture(monkeypatch):
                monkeypatch.setenv("HELLO", "WORLD")
        """)

        p = testdir.maketxtfile("""
            >>> import os
            >>> os.environ["HELLO"]
            'WORLD'
        """)
        reprec = testdir.inline_run(p, )
        reprec.assertoutcome(passed=1)

    def test_doctestmodule_with_fixtures(self, testdir):
        p = testdir.makepyfile("""
            '''
                >>> dir = getfixture('tmpdir')
                >>> type(dir).__name__
                'LocalPath'
            '''
        """)
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(passed=1)

    def test_doctestmodule_three_tests(self, testdir):
        p = testdir.makepyfile("""
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
        """)
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(passed=3)

    def test_doctestmodule_two_tests_one_fail(self, testdir):
        p = testdir.makepyfile("""
            class MyClass:
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
        """)
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(failed=1, passed=1)

    def test_ignored_whitespace(self, testdir):
        testdir.makeini("""
            [pytest]
            doctest_optionflags = ELLIPSIS NORMALIZE_WHITESPACE
        """)
        p = testdir.makepyfile("""
            class MyClass:
                '''
                >>> a = "foo    "
                >>> print(a)
                foo
                '''
                pass
        """)
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(passed=1)

    def test_non_ignored_whitespace(self, testdir):
        testdir.makeini("""
            [pytest]
            doctest_optionflags = ELLIPSIS
        """)
        p = testdir.makepyfile("""
            class MyClass:
                '''
                >>> a = "foo    "
                >>> print(a)
                foo
                '''
                pass
        """)
        reprec = testdir.inline_run(p, "--doctest-modules")
        reprec.assertoutcome(failed=1, passed=0)

    def test_ignored_whitespace_glob(self, testdir):
        testdir.makeini("""
            [pytest]
            doctest_optionflags = ELLIPSIS NORMALIZE_WHITESPACE
        """)
        p = testdir.maketxtfile(xdoc="""
            >>> a = "foo    "
            >>> print(a)
            foo
        """)
        reprec = testdir.inline_run(p, "--doctest-glob=x*.txt")
        reprec.assertoutcome(passed=1)

    def test_non_ignored_whitespace_glob(self, testdir):
        testdir.makeini("""
            [pytest]
            doctest_optionflags = ELLIPSIS
        """)
        p = testdir.maketxtfile(xdoc="""
            >>> a = "foo    "
            >>> print(a)
            foo
        """)
        reprec = testdir.inline_run(p, "--doctest-glob=x*.txt")
        reprec.assertoutcome(failed=1, passed=0)

    def test_contains_unicode(self, testdir):
        """Fix internal error with docstrings containing non-ascii characters.
        """
        testdir.makepyfile(u'''
            # encoding: utf-8
            def foo():
                """
                >>> name = 'с' # not letter 'c' but instead Cyrillic 's'.
                'anything'
                """
        ''')
        result = testdir.runpytest('--doctest-modules')
        result.stdout.fnmatch_lines([
            'Got nothing',
            '* 1 failed in*',
        ])

    def test_ignore_import_errors_on_doctest(self, testdir):
        p = testdir.makepyfile("""
            import asdf

            def add_one(x):
                '''
                >>> add_one(1)
                2
                '''
                return x + 1
        """)

        reprec = testdir.inline_run(p, "--doctest-modules",
                                    "--doctest-ignore-import-errors")
        reprec.assertoutcome(skipped=1, failed=1, passed=0)

    def test_junit_report_for_doctest(self, testdir):
        """
        #713: Fix --junit-xml option when used with --doctest-modules.
        """
        p = testdir.makepyfile("""
            def foo():
                '''
                >>> 1 + 1
                3
                '''
                pass
        """)
        reprec = testdir.inline_run(p, "--doctest-modules",
                                    "--junit-xml=junit.xml")
        reprec.assertoutcome(failed=1)


class TestLiterals:

    @pytest.mark.parametrize('config_mode', ['ini', 'comment'])
    def test_allow_unicode(self, testdir, config_mode):
        """Test that doctests which output unicode work in all python versions
        tested by pytest when the ALLOW_UNICODE option is used (either in
        the ini file or by an inline comment).
        """
        if config_mode == 'ini':
            testdir.makeini('''
            [pytest]
            doctest_optionflags = ALLOW_UNICODE
            ''')
            comment = ''
        else:
            comment = '#doctest: +ALLOW_UNICODE'

        testdir.maketxtfile(test_doc="""
            >>> b'12'.decode('ascii') {comment}
            '12'
        """.format(comment=comment))
        testdir.makepyfile(foo="""
            def foo():
              '''
              >>> b'12'.decode('ascii') {comment}
              '12'
              '''
        """.format(comment=comment))
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(passed=2)

    @pytest.mark.parametrize('config_mode', ['ini', 'comment'])
    def test_allow_bytes(self, testdir, config_mode):
        """Test that doctests which output bytes work in all python versions
        tested by pytest when the ALLOW_BYTES option is used (either in
        the ini file or by an inline comment)(#1287).
        """
        if config_mode == 'ini':
            testdir.makeini('''
            [pytest]
            doctest_optionflags = ALLOW_BYTES
            ''')
            comment = ''
        else:
            comment = '#doctest: +ALLOW_BYTES'

        testdir.maketxtfile(test_doc="""
            >>> b'foo'  {comment}
            'foo'
        """.format(comment=comment))
        testdir.makepyfile(foo="""
            def foo():
              '''
              >>> b'foo'  {comment}
              'foo'
              '''
        """.format(comment=comment))
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(passed=2)

    def test_unicode_string(self, testdir):
        """Test that doctests which output unicode fail in Python 2 when
        the ALLOW_UNICODE option is not used. The same test should pass
        in Python 3.
        """
        testdir.maketxtfile(test_doc="""
            >>> b'12'.decode('ascii')
            '12'
        """)
        reprec = testdir.inline_run()
        passed = int(sys.version_info[0] >= 3)
        reprec.assertoutcome(passed=passed, failed=int(not passed))

    def test_bytes_literal(self, testdir):
        """Test that doctests which output bytes fail in Python 3 when
        the ALLOW_BYTES option is not used. The same test should pass
        in Python 2 (#1287).
        """
        testdir.maketxtfile(test_doc="""
            >>> b'foo'
            'foo'
        """)
        reprec = testdir.inline_run()
        passed = int(sys.version_info[0] == 2)
        reprec.assertoutcome(passed=passed, failed=int(not passed))


class TestDoctestSkips:
    """
    If all examples in a doctest are skipped due to the SKIP option, then
    the tests should be SKIPPED rather than PASSED. (#957)
    """

    @pytest.fixture(params=['text', 'module'])
    def makedoctest(self, testdir, request):
        def makeit(doctest):
            mode = request.param
            if mode == 'text':
                testdir.maketxtfile(doctest)
            else:
                assert mode == 'module'
                testdir.makepyfile('"""\n%s"""' % doctest)

        return makeit

    def test_one_skipped(self, testdir, makedoctest):
        makedoctest("""
            >>> 1 + 1  # doctest: +SKIP
            2
            >>> 2 + 2
            4
        """)
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(passed=1)

    def test_one_skipped_failed(self, testdir, makedoctest):
        makedoctest("""
            >>> 1 + 1  # doctest: +SKIP
            2
            >>> 2 + 2
            200
        """)
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(failed=1)

    def test_all_skipped(self, testdir, makedoctest):
        makedoctest("""
            >>> 1 + 1  # doctest: +SKIP
            2
            >>> 2 + 2  # doctest: +SKIP
            200
        """)
        reprec = testdir.inline_run("--doctest-modules")
        reprec.assertoutcome(skipped=1)


class TestDoctestAutoUseFixtures:

    SCOPES = ['module', 'session', 'class', 'function']

    def test_doctest_module_session_fixture(self, testdir):
        """Test that session fixtures are initialized for doctest modules (#768)
        """
        # session fixture which changes some global data, which will
        # be accessed by doctests in a module
        testdir.makeconftest("""
            import pytest
            import sys

            @pytest.yield_fixture(autouse=True, scope='session')
            def myfixture():
                assert not hasattr(sys, 'pytest_session_data')
                sys.pytest_session_data = 1
                yield
                del sys.pytest_session_data
        """)
        testdir.makepyfile(foo="""
            import sys

            def foo():
              '''
              >>> assert sys.pytest_session_data == 1
              '''

            def bar():
              '''
              >>> assert sys.pytest_session_data == 1
              '''
        """)
        result = testdir.runpytest("--doctest-modules")
        result.stdout.fnmatch_lines('*2 passed*')

    @pytest.mark.parametrize('scope', SCOPES)
    @pytest.mark.parametrize('enable_doctest', [True, False])
    def test_fixture_scopes(self, testdir, scope, enable_doctest):
        """Test that auto-use fixtures work properly with doctest modules.
        See #1057 and #1100.
        """
        testdir.makeconftest('''
            import pytest

            @pytest.fixture(autouse=True, scope="{scope}")
            def auto(request):
                return 99
        '''.format(scope=scope))
        testdir.makepyfile(test_1='''
            def test_foo():
                """
                >>> getfixture('auto') + 1
                100
                """
            def test_bar():
                assert 1
        ''')
        params = ('--doctest-modules',) if enable_doctest else ()
        passes = 3 if enable_doctest else 2
        result = testdir.runpytest(*params)
        result.stdout.fnmatch_lines(['*=== %d passed in *' % passes])

    @pytest.mark.parametrize('scope', SCOPES)
    @pytest.mark.parametrize('autouse', [True, False])
    @pytest.mark.parametrize('use_fixture_in_doctest', [True, False])
    def test_fixture_module_doctest_scopes(self, testdir, scope, autouse,
                                           use_fixture_in_doctest):
        """Test that auto-use fixtures work properly with doctest files.
        See #1057 and #1100.
        """
        testdir.makeconftest('''
            import pytest

            @pytest.fixture(autouse={autouse}, scope="{scope}")
            def auto(request):
                return 99
        '''.format(scope=scope, autouse=autouse))
        if use_fixture_in_doctest:
            testdir.maketxtfile(test_doc="""
                >>> getfixture('auto')
                99
            """)
        else:
            testdir.maketxtfile(test_doc="""
                >>> 1 + 1
                2
            """)
        result = testdir.runpytest('--doctest-modules')
        assert 'FAILURES' not in str(result.stdout.str())
        result.stdout.fnmatch_lines(['*=== 1 passed in *'])

    @pytest.mark.parametrize('scope', SCOPES)
    def test_auto_use_request_attributes(self, testdir, scope):
        """Check that all attributes of a request in an autouse fixture
        behave as expected when requested for a doctest item.
        """
        testdir.makeconftest('''
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
        '''.format(scope=scope))
        testdir.maketxtfile(test_doc="""
            >>> 1 + 1
            2
        """)
        result = testdir.runpytest('--doctest-modules')
        assert 'FAILURES' not in str(result.stdout.str())
        result.stdout.fnmatch_lines(['*=== 1 passed in *'])
