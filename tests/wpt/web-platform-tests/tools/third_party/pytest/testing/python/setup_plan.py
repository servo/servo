# -*- coding: utf-8 -*-
def test_show_fixtures_and_test(testdir):
    """ Verifies that fixtures are not executed. """
    p = testdir.makepyfile(
        """
        import pytest
        @pytest.fixture
        def arg():
            assert False
        def test_arg(arg):
            assert False
    """
    )

    result = testdir.runpytest("--setup-plan", p)
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        ["*SETUP    F arg*", "*test_arg (fixtures used: arg)", "*TEARDOWN F arg*"]
    )
