#
# test correct setup/teardowns at
# module, class, and instance level

def test_module_and_function_setup(testdir):
    reprec = testdir.inline_runsource("""
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

        class TestFromClass:
            def test_module(self):
                assert modlevel[0] == 42
                assert not hasattr(test_modlevel, 'answer')
    """)
    rep = reprec.matchreport("test_modlevel")
    assert rep.passed
    rep = reprec.matchreport("test_module")
    assert rep.passed

def test_module_setup_failure_no_teardown(testdir):
    reprec = testdir.inline_runsource("""
        l = []
        def setup_module(module):
            l.append(1)
            0/0

        def test_nothing():
            pass

        def teardown_module(module):
            l.append(2)
    """)
    reprec.assertoutcome(failed=1)
    calls = reprec.getcalls("pytest_runtest_setup")
    assert calls[0].item.module.l == [1]

def test_setup_function_failure_no_teardown(testdir):
    reprec = testdir.inline_runsource("""
        modlevel = []
        def setup_function(function):
            modlevel.append(1)
            0/0

        def teardown_function(module):
            modlevel.append(2)

        def test_func():
            pass
    """)
    calls = reprec.getcalls("pytest_runtest_setup")
    assert calls[0].item.module.modlevel == [1]

def test_class_setup(testdir):
    reprec = testdir.inline_runsource("""
        class TestSimpleClassSetup:
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
    """)
    reprec.assertoutcome(passed=1+2+1)

def test_class_setup_failure_no_teardown(testdir):
    reprec = testdir.inline_runsource("""
        class TestSimpleClassSetup:
            clslevel = []
            def setup_class(cls):
                0/0

            def teardown_class(cls):
                cls.clslevel.append(1)

            def test_classlevel(self):
                pass

        def test_cleanup():
            assert not TestSimpleClassSetup.clslevel
    """)
    reprec.assertoutcome(failed=1, passed=1)

def test_method_setup(testdir):
    reprec = testdir.inline_runsource("""
        class TestSetupMethod:
            def setup_method(self, meth):
                self.methsetup = meth
            def teardown_method(self, meth):
                del self.methsetup

            def test_some(self):
                assert self.methsetup == self.test_some

            def test_other(self):
                assert self.methsetup == self.test_other
    """)
    reprec.assertoutcome(passed=2)

def test_method_setup_failure_no_teardown(testdir):
    reprec = testdir.inline_runsource("""
        class TestMethodSetup:
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
    """)
    reprec.assertoutcome(failed=1, passed=1)

def test_method_generator_setup(testdir):
    reprec = testdir.inline_runsource("""
        class TestSetupTeardownOnInstance:
            def setup_class(cls):
                cls.classsetup = True

            def setup_method(self, method):
                self.methsetup = method

            def test_generate(self):
                assert self.classsetup
                assert self.methsetup == self.test_generate
                yield self.generated, 5
                yield self.generated, 2

            def generated(self, value):
                assert self.classsetup
                assert self.methsetup == self.test_generate
                assert value == 5
    """)
    reprec.assertoutcome(passed=1, failed=1)

def test_func_generator_setup(testdir):
    reprec = testdir.inline_runsource("""
        import sys

        def setup_module(mod):
            print ("setup_module")
            mod.x = []

        def setup_function(fun):
            print ("setup_function")
            x.append(1)

        def teardown_function(fun):
            print ("teardown_function")
            x.pop()

        def test_one():
            assert x == [1]
            def check():
                print ("check")
                sys.stderr.write("e\\n")
                assert x == [1]
            yield check
            assert x == [1]
    """)
    rep = reprec.matchreport("test_one", names="pytest_runtest_logreport")
    assert rep.passed

def test_method_setup_uses_fresh_instances(testdir):
    reprec = testdir.inline_runsource("""
        class TestSelfState1:
            memory = []
            def test_hello(self):
                self.memory.append(self)

            def test_afterhello(self):
                assert self != self.memory[0]
    """)
    reprec.assertoutcome(passed=2, failed=0)

def test_setup_that_skips_calledagain(testdir):
    p = testdir.makepyfile("""
        import pytest
        def setup_module(mod):
            pytest.skip("x")
        def test_function1():
            pass
        def test_function2():
            pass
    """)
    reprec = testdir.inline_run(p)
    reprec.assertoutcome(skipped=2)

def test_setup_fails_again_on_all_tests(testdir):
    p = testdir.makepyfile("""
        import pytest
        def setup_module(mod):
            raise ValueError(42)
        def test_function1():
            pass
        def test_function2():
            pass
    """)
    reprec = testdir.inline_run(p)
    reprec.assertoutcome(failed=2)

def test_setup_funcarg_setup_when_outer_scope_fails(testdir):
    p = testdir.makepyfile("""
        import pytest
        def setup_module(mod):
            raise ValueError(42)
        def pytest_funcarg__hello(request):
            raise ValueError("xyz43")
        def test_function1(hello):
            pass
        def test_function2(hello):
            pass
    """)
    result = testdir.runpytest(p)
    result.stdout.fnmatch_lines([
        "*function1*",
        "*ValueError*42*",
        "*function2*",
        "*ValueError*42*",
        "*2 error*"
    ])
    assert "xyz43" not in result.stdout.str()
