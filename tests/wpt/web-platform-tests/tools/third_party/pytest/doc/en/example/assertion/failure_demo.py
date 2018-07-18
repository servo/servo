from pytest import raises
import _pytest._code
import py


def otherfunc(a, b):
    assert a == b


def somefunc(x, y):
    otherfunc(x, y)


def otherfunc_multi(a, b):
    assert a == b


def test_generative(param1, param2):
    assert param1 * 2 < param2


def pytest_generate_tests(metafunc):
    if "param1" in metafunc.fixturenames:
        metafunc.addcall(funcargs=dict(param1=3, param2=6))


class TestFailing(object):

    def test_simple(self):

        def f():
            return 42

        def g():
            return 43

        assert f() == g()

    def test_simple_multiline(self):
        otherfunc_multi(42, 6 * 9)

    def test_not(self):

        def f():
            return 42

        assert not f()


class TestSpecialisedExplanations(object):

    def test_eq_text(self):
        assert "spam" == "eggs"

    def test_eq_similar_text(self):
        assert "foo 1 bar" == "foo 2 bar"

    def test_eq_multiline_text(self):
        assert "foo\nspam\nbar" == "foo\neggs\nbar"

    def test_eq_long_text(self):
        a = "1" * 100 + "a" + "2" * 100
        b = "1" * 100 + "b" + "2" * 100
        assert a == b

    def test_eq_long_text_multiline(self):
        a = "1\n" * 100 + "a" + "2\n" * 100
        b = "1\n" * 100 + "b" + "2\n" * 100
        assert a == b

    def test_eq_list(self):
        assert [0, 1, 2] == [0, 1, 3]

    def test_eq_list_long(self):
        a = [0] * 100 + [1] + [3] * 100
        b = [0] * 100 + [2] + [3] * 100
        assert a == b

    def test_eq_dict(self):
        assert {"a": 0, "b": 1, "c": 0} == {"a": 0, "b": 2, "d": 0}

    def test_eq_set(self):
        assert {0, 10, 11, 12} == {0, 20, 21}

    def test_eq_longer_list(self):
        assert [1, 2] == [1, 2, 3]

    def test_in_list(self):
        assert 1 in [0, 2, 3, 4, 5]

    def test_not_in_text_multiline(self):
        text = "some multiline\ntext\nwhich\nincludes foo\nand a\ntail"
        assert "foo" not in text

    def test_not_in_text_single(self):
        text = "single foo line"
        assert "foo" not in text

    def test_not_in_text_single_long(self):
        text = "head " * 50 + "foo " + "tail " * 20
        assert "foo" not in text

    def test_not_in_text_single_long_term(self):
        text = "head " * 50 + "f" * 70 + "tail " * 20
        assert "f" * 70 not in text


def test_attribute():

    class Foo(object):
        b = 1

    i = Foo()
    assert i.b == 2


def test_attribute_instance():

    class Foo(object):
        b = 1

    assert Foo().b == 2


def test_attribute_failure():

    class Foo(object):

        def _get_b(self):
            raise Exception("Failed to get attrib")

        b = property(_get_b)

    i = Foo()
    assert i.b == 2


def test_attribute_multiple():

    class Foo(object):
        b = 1

    class Bar(object):
        b = 2

    assert Foo().b == Bar().b


def globf(x):
    return x + 1


class TestRaises(object):

    def test_raises(self):
        s = "qwe"  # NOQA
        raises(TypeError, "int(s)")

    def test_raises_doesnt(self):
        raises(IOError, "int('3')")

    def test_raise(self):
        raise ValueError("demo error")

    def test_tupleerror(self):
        a, b = [1]  # NOQA

    def test_reinterpret_fails_with_print_for_the_fun_of_it(self):
        items = [1, 2, 3]
        print("items is %r" % items)
        a, b = items.pop()

    def test_some_error(self):
        if namenotexi:  # NOQA
            pass

    def func1(self):
        assert 41 == 42


# thanks to Matthew Scott for this test
def test_dynamic_compile_shows_nicely():
    import imp
    import sys

    src = "def foo():\n assert 1 == 0\n"
    name = "abc-123"
    module = imp.new_module(name)
    code = _pytest._code.compile(src, name, "exec")
    py.builtin.exec_(code, module.__dict__)
    sys.modules[name] = module
    module.foo()


class TestMoreErrors(object):

    def test_complex_error(self):

        def f():
            return 44

        def g():
            return 43

        somefunc(f(), g())

    def test_z1_unpack_error(self):
        items = []
        a, b = items

    def test_z2_type_error(self):
        items = 3
        a, b = items

    def test_startswith(self):
        s = "123"
        g = "456"
        assert s.startswith(g)

    def test_startswith_nested(self):

        def f():
            return "123"

        def g():
            return "456"

        assert f().startswith(g())

    def test_global_func(self):
        assert isinstance(globf(42), float)

    def test_instance(self):
        self.x = 6 * 7
        assert self.x != 42

    def test_compare(self):
        assert globf(10) < 5

    def test_try_finally(self):
        x = 1
        try:
            assert x == 0
        finally:
            x = 0


class TestCustomAssertMsg(object):

    def test_single_line(self):

        class A(object):
            a = 1

        b = 2
        assert A.a == b, "A.a appears not to be b"

    def test_multiline(self):

        class A(object):
            a = 1

        b = 2
        assert (
            A.a == b
        ), "A.a appears not to be b\n" "or does not appear to be b\none of those"

    def test_custom_repr(self):

        class JSON(object):
            a = 1

            def __repr__(self):
                return "This is JSON\n{\n  'foo': 'bar'\n}"

        a = JSON()
        b = 2
        assert a.a == b, a
