# mypy: allow-untyped-defs
"""Test correct setup/teardowns at module, class, and instance level."""

from typing import List

from _pytest.pytester import Pytester
import pytest


def test_module_and_function_setup(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        modlevel = []
        def setup_module(module):
            assert not modlevel
            module.modlevel.append(42)

        def teardown_module(module):
            modlevel.pop()

        def setup_function(function):
            function.answer = 17

        def teardown_function(function):
            del function.answer

        def test_modlevel():
            assert modlevel[0] == 42
            assert test_modlevel.answer == 17

        class TestFromClass(object):
            def test_module(self):
                assert modlevel[0] == 42
                assert not hasattr(test_modlevel, 'answer')
    """
    )
    rep = reprec.matchreport("test_modlevel")
    assert rep.passed
    rep = reprec.matchreport("test_module")
    assert rep.passed


def test_module_setup_failure_no_teardown(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        values = []
        def setup_module(module):
            values.append(1)
            0/0

        def test_nothing():
            pass

        def teardown_module(module):
            values.append(2)
    """
    )
    reprec.assertoutcome(failed=1)
    calls = reprec.getcalls("pytest_runtest_setup")
    assert calls[0].item.module.values == [1]


def test_setup_function_failure_no_teardown(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        modlevel = []
        def setup_function(function):
            modlevel.append(1)
            0/0

        def teardown_function(module):
            modlevel.append(2)

        def test_func():
            pass
    """
    )
    calls = reprec.getcalls("pytest_runtest_setup")
    assert calls[0].item.module.modlevel == [1]


def test_class_setup(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        class TestSimpleClassSetup(object):
            clslevel = []
            def setup_class(cls):
                cls.clslevel.append(23)

            def teardown_class(cls):
                cls.clslevel.pop()

            def test_classlevel(self):
                assert self.clslevel[0] == 23

        class TestInheritedClassSetupStillWorks(TestSimpleClassSetup):
            def test_classlevel_anothertime(self):
                assert self.clslevel == [23]

        def test_cleanup():
            assert not TestSimpleClassSetup.clslevel
            assert not TestInheritedClassSetupStillWorks.clslevel
    """
    )
    reprec.assertoutcome(passed=1 + 2 + 1)


def test_class_setup_failure_no_teardown(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        class TestSimpleClassSetup(object):
            clslevel = []
            def setup_class(cls):
                0/0

            def teardown_class(cls):
                cls.clslevel.append(1)

            def test_classlevel(self):
                pass

        def test_cleanup():
            assert not TestSimpleClassSetup.clslevel
    """
    )
    reprec.assertoutcome(failed=1, passed=1)


def test_method_setup(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        class TestSetupMethod(object):
            def setup_method(self, meth):
                self.methsetup = meth
            def teardown_method(self, meth):
                del self.methsetup

            def test_some(self):
                assert self.methsetup == self.test_some

            def test_other(self):
                assert self.methsetup == self.test_other
    """
    )
    reprec.assertoutcome(passed=2)


def test_method_setup_failure_no_teardown(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        class TestMethodSetup(object):
            clslevel = []
            def setup_method(self, method):
                self.clslevel.append(1)
                0/0

            def teardown_method(self, method):
                self.clslevel.append(2)

            def test_method(self):
                pass

        def test_cleanup():
            assert TestMethodSetup.clslevel == [1]
    """
    )
    reprec.assertoutcome(failed=1, passed=1)


def test_method_setup_uses_fresh_instances(pytester: Pytester) -> None:
    reprec = pytester.inline_runsource(
        """
        class TestSelfState1(object):
            memory = []
            def test_hello(self):
                self.memory.append(self)

            def test_afterhello(self):
                assert self != self.memory[0]
    """
    )
    reprec.assertoutcome(passed=2, failed=0)


def test_setup_that_skips_calledagain(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest
        def setup_module(mod):
            pytest.skip("x")
        def test_function1():
            pass
        def test_function2():
            pass
    """
    )
    reprec = pytester.inline_run(p)
    reprec.assertoutcome(skipped=2)


def test_setup_fails_again_on_all_tests(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest
        def setup_module(mod):
            raise ValueError(42)
        def test_function1():
            pass
        def test_function2():
            pass
    """
    )
    reprec = pytester.inline_run(p)
    reprec.assertoutcome(failed=2)


def test_setup_funcarg_setup_when_outer_scope_fails(pytester: Pytester) -> None:
    p = pytester.makepyfile(
        """
        import pytest
        def setup_module(mod):
            raise ValueError(42)
        @pytest.fixture
        def hello(request):
            raise ValueError("xyz43")
        def test_function1(hello):
            pass
        def test_function2(hello):
            pass
    """
    )
    result = pytester.runpytest(p)
    result.stdout.fnmatch_lines(
        [
            "*function1*",
            "*ValueError*42*",
            "*function2*",
            "*ValueError*42*",
            "*2 errors*",
        ]
    )
    result.stdout.no_fnmatch_line("*xyz43*")


@pytest.mark.parametrize("arg", ["", "arg"])
def test_setup_teardown_function_level_with_optional_argument(
    pytester: Pytester,
    monkeypatch,
    arg: str,
) -> None:
    """Parameter to setup/teardown xunit-style functions parameter is now optional (#1728)."""
    import sys

    trace_setups_teardowns: List[str] = []
    monkeypatch.setattr(
        sys, "trace_setups_teardowns", trace_setups_teardowns, raising=False
    )
    p = pytester.makepyfile(
        f"""
        import pytest
        import sys

        trace = sys.trace_setups_teardowns.append

        def setup_module({arg}): trace('setup_module')
        def teardown_module({arg}): trace('teardown_module')

        def setup_function({arg}): trace('setup_function')
        def teardown_function({arg}): trace('teardown_function')

        def test_function_1(): pass
        def test_function_2(): pass

        class Test(object):
            def setup_method(self, {arg}): trace('setup_method')
            def teardown_method(self, {arg}): trace('teardown_method')

            def test_method_1(self): pass
            def test_method_2(self): pass
    """
    )
    result = pytester.inline_run(p)
    result.assertoutcome(passed=4)

    expected = [
        "setup_module",
        "setup_function",
        "teardown_function",
        "setup_function",
        "teardown_function",
        "setup_method",
        "teardown_method",
        "setup_method",
        "teardown_method",
        "teardown_module",
    ]
    assert trace_setups_teardowns == expected
