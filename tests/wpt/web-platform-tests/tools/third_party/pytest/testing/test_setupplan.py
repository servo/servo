def test_show_fixtures_and_test(testdir, dummy_yaml_custom_test):
    """Verify that fixtures are not executed."""
    testdir.makepyfile(
        """
        import pytest
        @pytest.fixture
        def arg():
            assert False
        def test_arg(arg):
            assert False
    """
    )

    result = testdir.runpytest("--setup-plan")
    assert result.ret == 0

    result.stdout.fnmatch_lines(
        ["*SETUP    F arg*", "*test_arg (fixtures used: arg)", "*TEARDOWN F arg*"]
    )


def test_show_multi_test_fixture_setup_and_teardown_correctly_simple(testdir):
    """Verify that when a fixture lives for longer than a single test, --setup-plan
    correctly displays the SETUP/TEARDOWN indicators the right number of times.

    As reported in https://github.com/pytest-dev/pytest/issues/2049
    --setup-plan was showing SETUP/TEARDOWN on every test, even when the fixture
    should persist through multiple tests.

    (Note that this bug never affected actual test execution, which used the
    correct fixture lifetimes. It was purely a display bug for --setup-plan, and
    did not affect the related --setup-show or --setup-only.)
    """
    testdir.makepyfile(
        """
        import pytest
        @pytest.fixture(scope = 'class')
        def fix():
            return object()
        class TestClass:
            def test_one(self, fix):
                assert False
            def test_two(self, fix):
                assert False
    """
    )

    result = testdir.runpytest("--setup-plan")
    assert result.ret == 0

    setup_fragment = "SETUP    C fix"
    setup_count = 0

    teardown_fragment = "TEARDOWN C fix"
    teardown_count = 0

    for line in result.stdout.lines:
        if setup_fragment in line:
            setup_count += 1
        if teardown_fragment in line:
            teardown_count += 1

    # before the fix this tests, there would have been a setup/teardown
    # message for each test, so the counts would each have been 2
    assert setup_count == 1
    assert teardown_count == 1


def test_show_multi_test_fixture_setup_and_teardown_same_as_setup_show(testdir):
    """Verify that SETUP/TEARDOWN messages match what comes out of --setup-show."""
    testdir.makepyfile(
        """
        import pytest
        @pytest.fixture(scope = 'session')
        def sess():
            return True
        @pytest.fixture(scope = 'module')
        def mod():
            return True
        @pytest.fixture(scope = 'class')
        def cls():
            return True
        @pytest.fixture(scope = 'function')
        def func():
            return True
        def test_outside(sess, mod, cls, func):
            assert True
        class TestCls:
            def test_one(self, sess, mod, cls, func):
                assert True
            def test_two(self, sess, mod, cls, func):
                assert True
    """
    )

    plan_result = testdir.runpytest("--setup-plan")
    show_result = testdir.runpytest("--setup-show")

    # the number and text of these lines should be identical
    plan_lines = [
        line
        for line in plan_result.stdout.lines
        if "SETUP" in line or "TEARDOWN" in line
    ]
    show_lines = [
        line
        for line in show_result.stdout.lines
        if "SETUP" in line or "TEARDOWN" in line
    ]

    assert plan_lines == show_lines
