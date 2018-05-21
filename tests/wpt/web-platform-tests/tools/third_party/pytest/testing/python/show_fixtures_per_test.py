# -*- coding: utf-8 -*-


def test_no_items_should_not_show_output(testdir):
    result = testdir.runpytest('--fixtures-per-test')
    assert 'fixtures used by' not in result.stdout.str()
    assert result.ret == 0


def test_fixtures_in_module(testdir):
    p = testdir.makepyfile('''
        import pytest
        @pytest.fixture
        def _arg0():
            """hidden arg0 fixture"""
        @pytest.fixture
        def arg1():
            """arg1 docstring"""
        def test_arg1(arg1):
            pass
    ''')

    result = testdir.runpytest("--fixtures-per-test", p)
    assert result.ret == 0

    result.stdout.fnmatch_lines([
        '*fixtures used by test_arg1*',
        '*(test_fixtures_in_module.py:9)*',
        'arg1',
        '    arg1 docstring',
    ])
    assert "_arg0" not in result.stdout.str()


def test_fixtures_in_conftest(testdir):
    testdir.makeconftest('''
        import pytest
        @pytest.fixture
        def arg1():
            """arg1 docstring"""
        @pytest.fixture
        def arg2():
            """arg2 docstring"""
        @pytest.fixture
        def arg3(arg1, arg2):
            """arg3
            docstring
            """
    ''')
    p = testdir.makepyfile('''
        def test_arg2(arg2):
            pass
        def test_arg3(arg3):
            pass
    ''')
    result = testdir.runpytest("--fixtures-per-test", p)
    assert result.ret == 0

    result.stdout.fnmatch_lines([
        '*fixtures used by test_arg2*',
        '*(test_fixtures_in_conftest.py:2)*',
        'arg2',
        '    arg2 docstring',
        '*fixtures used by test_arg3*',
        '*(test_fixtures_in_conftest.py:4)*',
        'arg1',
        '    arg1 docstring',
        'arg2',
        '    arg2 docstring',
        'arg3',
        '    arg3',
        '    docstring',
    ])


def test_should_show_fixtures_used_by_test(testdir):
    testdir.makeconftest('''
        import pytest
        @pytest.fixture
        def arg1():
            """arg1 from conftest"""
        @pytest.fixture
        def arg2():
            """arg2 from conftest"""
    ''')
    p = testdir.makepyfile('''
        import pytest
        @pytest.fixture
        def arg1():
            """arg1 from testmodule"""
        def test_args(arg1, arg2):
            pass
    ''')
    result = testdir.runpytest("--fixtures-per-test", p)
    assert result.ret == 0

    result.stdout.fnmatch_lines([
        '*fixtures used by test_args*',
        '*(test_should_show_fixtures_used_by_test.py:6)*',
        'arg1',
        '    arg1 from testmodule',
        'arg2',
        '    arg2 from conftest',
    ])


def test_verbose_include_private_fixtures_and_loc(testdir):
    testdir.makeconftest('''
        import pytest
        @pytest.fixture
        def _arg1():
            """_arg1 from conftest"""
        @pytest.fixture
        def arg2(_arg1):
            """arg2 from conftest"""
    ''')
    p = testdir.makepyfile('''
        import pytest
        @pytest.fixture
        def arg3():
            """arg3 from testmodule"""
        def test_args(arg2, arg3):
            pass
    ''')
    result = testdir.runpytest("--fixtures-per-test", "-v", p)
    assert result.ret == 0

    result.stdout.fnmatch_lines([
        '*fixtures used by test_args*',
        '*(test_verbose_include_private_fixtures_and_loc.py:6)*',
        '_arg1 -- conftest.py:3',
        '    _arg1 from conftest',
        'arg2 -- conftest.py:6',
        '    arg2 from conftest',
        'arg3 -- test_verbose_include_private_fixtures_and_loc.py:3',
        '    arg3 from testmodule',
    ])


def test_doctest_items(testdir):
    testdir.makepyfile('''
        def foo():
            """
            >>> 1 + 1
            2
            """
    ''')
    testdir.maketxtfile('''
        >>> 1 + 1
        2
    ''')
    result = testdir.runpytest("--fixtures-per-test", "--doctest-modules",
                               "--doctest-glob=*.txt", "-v")
    assert result.ret == 0

    result.stdout.fnmatch_lines([
        '*collected 2 items*',
    ])
