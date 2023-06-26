import sys

import pytest
from _pytest.config import ExitCode
from _pytest.pytester import Pytester


@pytest.fixture(params=["--setup-only", "--setup-plan", "--setup-show"], scope="module")
def mode(request):
    return request.param


def test_show_only_active_fixtures(
    pytester: Pytester, mode, dummy_yaml_custom_test
) -> None:
    pytester.makepyfile(
        '''
        import pytest
        @pytest.fixture
        def _arg0():
            """hidden arg0 fixture"""
        @pytest.fixture
        def arg1():
            """arg1 docstring"""
        def test_arg1(arg1):
            pass
    '''
    )

    result = pytester.runpytest(mode)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        ["*SETUP    F arg1*", "*test_arg1 (fixtures used: arg1)*", "*TEARDOWN F arg1*"]
    )
    result.stdout.no_fnmatch_line("*_arg0*")


def test_show_different_scopes(pytester: Pytester, mode) -> None:
    p = pytester.makepyfile(
        '''
        import pytest
        @pytest.fixture
        def arg_function():
            """function scoped fixture"""
        @pytest.fixture(scope='session')
        def arg_session():
            """session scoped fixture"""
        def test_arg1(arg_session, arg_function):
            pass
    '''
    )

    result = pytester.runpytest(mode, p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        [
            "SETUP    S arg_session*",
            "*SETUP    F arg_function*",
            "*test_arg1 (fixtures used: arg_function, arg_session)*",
            "*TEARDOWN F arg_function*",
            "TEARDOWN S arg_session*",
        ]
    )


def test_show_nested_fixtures(pytester: Pytester, mode) -> None:
    pytester.makeconftest(
        '''
        import pytest
        @pytest.fixture(scope='session')
        def arg_same():
            """session scoped fixture"""
        '''
    )
    p = pytester.makepyfile(
        '''
        import pytest
        @pytest.fixture(scope='function')
        def arg_same(arg_same):
            """function scoped fixture"""
        def test_arg1(arg_same):
            pass
    '''
    )

    result = pytester.runpytest(mode, p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        [
            "SETUP    S arg_same*",
            "*SETUP    F arg_same (fixtures used: arg_same)*",
            "*test_arg1 (fixtures used: arg_same)*",
            "*TEARDOWN F arg_same*",
            "TEARDOWN S arg_same*",
        ]
    )


def test_show_fixtures_with_autouse(pytester: Pytester, mode) -> None:
    p = pytester.makepyfile(
        '''
        import pytest
        @pytest.fixture
        def arg_function():
            """function scoped fixture"""
        @pytest.fixture(scope='session', autouse=True)
        def arg_session():
            """session scoped fixture"""
        def test_arg1(arg_function):
            pass
    '''
    )

    result = pytester.runpytest(mode, p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        [
            "SETUP    S arg_session*",
            "*SETUP    F arg_function*",
            "*test_arg1 (fixtures used: arg_function, arg_session)*",
        ]
    )


def test_show_fixtures_with_parameters(pytester: Pytester, mode) -> None:
    pytester.makeconftest(
        '''
        import pytest
        @pytest.fixture(scope='session', params=['foo', 'bar'])
        def arg_same():
            """session scoped fixture"""
        '''
    )
    p = pytester.makepyfile(
        '''
        import pytest
        @pytest.fixture(scope='function')
        def arg_other(arg_same):
            """function scoped fixture"""
        def test_arg1(arg_other):
            pass
    '''
    )

    result = pytester.runpytest(mode, p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        [
            "SETUP    S arg_same?'foo'?",
            "TEARDOWN S arg_same?'foo'?",
            "SETUP    S arg_same?'bar'?",
            "TEARDOWN S arg_same?'bar'?",
        ]
    )


def test_show_fixtures_with_parameter_ids(pytester: Pytester, mode) -> None:
    pytester.makeconftest(
        '''
        import pytest
        @pytest.fixture(
            scope='session', params=['foo', 'bar'], ids=['spam', 'ham'])
        def arg_same():
            """session scoped fixture"""
        '''
    )
    p = pytester.makepyfile(
        '''
        import pytest
        @pytest.fixture(scope='function')
        def arg_other(arg_same):
            """function scoped fixture"""
        def test_arg1(arg_other):
            pass
    '''
    )

    result = pytester.runpytest(mode, p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        ["SETUP    S arg_same?'spam'?", "SETUP    S arg_same?'ham'?"]
    )


def test_show_fixtures_with_parameter_ids_function(pytester: Pytester, mode) -> None:
    p = pytester.makepyfile(
        """
        import pytest
        @pytest.fixture(params=['foo', 'bar'], ids=lambda p: p.upper())
        def foobar():
            pass
        def test_foobar(foobar):
            pass
    """
    )

    result = pytester.runpytest(mode, p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        ["*SETUP    F foobar?'FOO'?", "*SETUP    F foobar?'BAR'?"]
    )


def test_dynamic_fixture_request(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest
        @pytest.fixture()
        def dynamically_requested_fixture():
            pass
        @pytest.fixture()
        def dependent_fixture(request):
            request.getfixturevalue('dynamically_requested_fixture')
        def test_dyn(dependent_fixture):
            pass
    """
    )

    result = pytester.runpytest("--setup-only", p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        [
            "*SETUP    F dynamically_requested_fixture",
            "*TEARDOWN F dynamically_requested_fixture",
        ]
    )


def test_capturing(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest, sys
        @pytest.fixture()
        def one():
            sys.stdout.write('this should be captured')
            sys.stderr.write('this should also be captured')
        @pytest.fixture()
        def two(one):
            assert 0
        def test_capturing(two):
            pass
    """
    )

    result = pytester.runpytest("--setup-only", p)
    result.stdout.fnmatch_lines(
        ["this should be captured", "this should also be captured"]
    )


def test_show_fixtures_and_execute_test(pytester: Pytester) -> None:
    """Verify that setups are shown and tests are executed."""
    p = pytester.makepyfile(
        """
        import pytest
        @pytest.fixture
        def arg():
            assert True
        def test_arg(arg):
            assert False
    """
    )

    result = pytester.runpytest("--setup-show", p)
    assert result.ret == 1

    result.stdout.fnmatch_lines(
        ["*SETUP    F arg*", "*test_arg (fixtures used: arg)F*", "*TEARDOWN F arg*"]
    )


def test_setup_show_with_KeyboardInterrupt_in_test(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest
        @pytest.fixture
        def arg():
            pass
        def test_arg(arg):
            raise KeyboardInterrupt()
    """
    )
    result = pytester.runpytest("--setup-show", p, no_reraise_ctrlc=True)
    result.stdout.fnmatch_lines(
        [
            "*SETUP    F arg*",
            "*test_arg (fixtures used: arg)*",
            "*TEARDOWN F arg*",
            "*! KeyboardInterrupt !*",
            "*= no tests ran in *",
        ]
    )
    assert result.ret == ExitCode.INTERRUPTED


def test_show_fixture_action_with_bytes(pytester: Pytester) -> None:
    # Issue 7126, BytesWarning when using --setup-show with bytes parameter
    test_file = pytester.makepyfile(
        """
        import pytest

        @pytest.mark.parametrize('data', [b'Hello World'])
        def test_data(data):
            pass
        """
    )
    result = pytester.run(
        sys.executable, "-bb", "-m", "pytest", "--setup-show", str(test_file)
    )
    assert result.ret == 0
