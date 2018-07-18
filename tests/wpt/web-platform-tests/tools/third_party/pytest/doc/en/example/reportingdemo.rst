
.. _`tbreportdemo`:

Demo of Python failure reports with pytest
==================================================

Here is a nice run of several tens of failures
and how ``pytest`` presents things (unfortunately
not showing the nice colors here in the HTML that you
get on the terminal - we are working on that)::

    assertion $ pytest failure_demo.py
    =========================== test session starts ============================
    platform linux -- Python 3.x.y, pytest-3.x.y, py-1.x.y, pluggy-0.x.y
    rootdir: $REGENDOC_TMPDIR/assertion, inifile:
    collected 42 items

    failure_demo.py FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF           [100%]

    ================================= FAILURES =================================
    ____________________________ test_generative[0] ____________________________

    param1 = 3, param2 = 6

        def test_generative(param1, param2):
    >       assert param1 * 2 < param2
    E       assert (3 * 2) < 6

    failure_demo.py:19: AssertionError
    _________________________ TestFailing.test_simple __________________________

    self = <failure_demo.TestFailing object at 0xdeadbeef>

        def test_simple(self):

            def f():
                return 42

            def g():
                return 43

    >       assert f() == g()
    E       assert 42 == 43
    E        +  where 42 = <function TestFailing.test_simple.<locals>.f at 0xdeadbeef>()
    E        +  and   43 = <function TestFailing.test_simple.<locals>.g at 0xdeadbeef>()

    failure_demo.py:37: AssertionError
    ____________________ TestFailing.test_simple_multiline _____________________

    self = <failure_demo.TestFailing object at 0xdeadbeef>

        def test_simple_multiline(self):
    >       otherfunc_multi(42, 6 * 9)

    failure_demo.py:40:
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

    a = 42, b = 54

        def otherfunc_multi(a, b):
    >       assert a == b
    E       assert 42 == 54

    failure_demo.py:15: AssertionError
    ___________________________ TestFailing.test_not ___________________________

    self = <failure_demo.TestFailing object at 0xdeadbeef>

        def test_not(self):

            def f():
                return 42

    >       assert not f()
    E       assert not 42
    E        +  where 42 = <function TestFailing.test_not.<locals>.f at 0xdeadbeef>()

    failure_demo.py:47: AssertionError
    _________________ TestSpecialisedExplanations.test_eq_text _________________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_text(self):
    >       assert "spam" == "eggs"
    E       AssertionError: assert 'spam' == 'eggs'
    E         - spam
    E         + eggs

    failure_demo.py:53: AssertionError
    _____________ TestSpecialisedExplanations.test_eq_similar_text _____________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_similar_text(self):
    >       assert "foo 1 bar" == "foo 2 bar"
    E       AssertionError: assert 'foo 1 bar' == 'foo 2 bar'
    E         - foo 1 bar
    E         ?     ^
    E         + foo 2 bar
    E         ?     ^

    failure_demo.py:56: AssertionError
    ____________ TestSpecialisedExplanations.test_eq_multiline_text ____________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_multiline_text(self):
    >       assert "foo\nspam\nbar" == "foo\neggs\nbar"
    E       AssertionError: assert 'foo\nspam\nbar' == 'foo\neggs\nbar'
    E           foo
    E         - spam
    E         + eggs
    E           bar

    failure_demo.py:59: AssertionError
    ______________ TestSpecialisedExplanations.test_eq_long_text _______________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_long_text(self):
            a = "1" * 100 + "a" + "2" * 100
            b = "1" * 100 + "b" + "2" * 100
    >       assert a == b
    E       AssertionError: assert '111111111111...2222222222222' == '1111111111111...2222222222222'
    E         Skipping 90 identical leading characters in diff, use -v to show
    E         Skipping 91 identical trailing characters in diff, use -v to show
    E         - 1111111111a222222222
    E         ?           ^
    E         + 1111111111b222222222
    E         ?           ^

    failure_demo.py:64: AssertionError
    _________ TestSpecialisedExplanations.test_eq_long_text_multiline __________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_long_text_multiline(self):
            a = "1\n" * 100 + "a" + "2\n" * 100
            b = "1\n" * 100 + "b" + "2\n" * 100
    >       assert a == b
    E       AssertionError: assert '1\n1\n1\n1\n...n2\n2\n2\n2\n' == '1\n1\n1\n1\n1...n2\n2\n2\n2\n'
    E         Skipping 190 identical leading characters in diff, use -v to show
    E         Skipping 191 identical trailing characters in diff, use -v to show
    E           1
    E           1
    E           1
    E           1
    E           1...
    E
    E         ...Full output truncated (7 lines hidden), use '-vv' to show

    failure_demo.py:69: AssertionError
    _________________ TestSpecialisedExplanations.test_eq_list _________________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_list(self):
    >       assert [0, 1, 2] == [0, 1, 3]
    E       assert [0, 1, 2] == [0, 1, 3]
    E         At index 2 diff: 2 != 3
    E         Use -v to get the full diff

    failure_demo.py:72: AssertionError
    ______________ TestSpecialisedExplanations.test_eq_list_long _______________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_list_long(self):
            a = [0] * 100 + [1] + [3] * 100
            b = [0] * 100 + [2] + [3] * 100
    >       assert a == b
    E       assert [0, 0, 0, 0, 0, 0, ...] == [0, 0, 0, 0, 0, 0, ...]
    E         At index 100 diff: 1 != 2
    E         Use -v to get the full diff

    failure_demo.py:77: AssertionError
    _________________ TestSpecialisedExplanations.test_eq_dict _________________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_dict(self):
    >       assert {"a": 0, "b": 1, "c": 0} == {"a": 0, "b": 2, "d": 0}
    E       AssertionError: assert {'a': 0, 'b': 1, 'c': 0} == {'a': 0, 'b': 2, 'd': 0}
    E         Omitting 1 identical items, use -vv to show
    E         Differing items:
    E         {'b': 1} != {'b': 2}
    E         Left contains more items:
    E         {'c': 0}
    E         Right contains more items:
    E         {'d': 0}...
    E
    E         ...Full output truncated (2 lines hidden), use '-vv' to show

    failure_demo.py:80: AssertionError
    _________________ TestSpecialisedExplanations.test_eq_set __________________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_set(self):
    >       assert {0, 10, 11, 12} == {0, 20, 21}
    E       AssertionError: assert {0, 10, 11, 12} == {0, 20, 21}
    E         Extra items in the left set:
    E         10
    E         11
    E         12
    E         Extra items in the right set:
    E         20
    E         21...
    E
    E         ...Full output truncated (2 lines hidden), use '-vv' to show

    failure_demo.py:83: AssertionError
    _____________ TestSpecialisedExplanations.test_eq_longer_list ______________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_eq_longer_list(self):
    >       assert [1, 2] == [1, 2, 3]
    E       assert [1, 2] == [1, 2, 3]
    E         Right contains more items, first extra item: 3
    E         Use -v to get the full diff

    failure_demo.py:86: AssertionError
    _________________ TestSpecialisedExplanations.test_in_list _________________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_in_list(self):
    >       assert 1 in [0, 2, 3, 4, 5]
    E       assert 1 in [0, 2, 3, 4, 5]

    failure_demo.py:89: AssertionError
    __________ TestSpecialisedExplanations.test_not_in_text_multiline __________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_not_in_text_multiline(self):
            text = "some multiline\ntext\nwhich\nincludes foo\nand a\ntail"
    >       assert "foo" not in text
    E       AssertionError: assert 'foo' not in 'some multiline\ntext\nw...ncludes foo\nand a\ntail'
    E         'foo' is contained here:
    E           some multiline
    E           text
    E           which
    E           includes foo
    E         ?          +++
    E           and a...
    E
    E         ...Full output truncated (2 lines hidden), use '-vv' to show

    failure_demo.py:93: AssertionError
    ___________ TestSpecialisedExplanations.test_not_in_text_single ____________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_not_in_text_single(self):
            text = "single foo line"
    >       assert "foo" not in text
    E       AssertionError: assert 'foo' not in 'single foo line'
    E         'foo' is contained here:
    E           single foo line
    E         ?        +++

    failure_demo.py:97: AssertionError
    _________ TestSpecialisedExplanations.test_not_in_text_single_long _________

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_not_in_text_single_long(self):
            text = "head " * 50 + "foo " + "tail " * 20
    >       assert "foo" not in text
    E       AssertionError: assert 'foo' not in 'head head head head hea...ail tail tail tail tail '
    E         'foo' is contained here:
    E           head head foo tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail
    E         ?           +++

    failure_demo.py:101: AssertionError
    ______ TestSpecialisedExplanations.test_not_in_text_single_long_term _______

    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>

        def test_not_in_text_single_long_term(self):
            text = "head " * 50 + "f" * 70 + "tail " * 20
    >       assert "f" * 70 not in text
    E       AssertionError: assert 'fffffffffff...ffffffffffff' not in 'head head he...l tail tail '
    E         'ffffffffffffffffff...fffffffffffffffffff' is contained here:
    E           head head fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffftail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail
    E         ?           ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++

    failure_demo.py:105: AssertionError
    ______________________________ test_attribute ______________________________

        def test_attribute():

            class Foo(object):
                b = 1

            i = Foo()
    >       assert i.b == 2
    E       assert 1 == 2
    E        +  where 1 = <failure_demo.test_attribute.<locals>.Foo object at 0xdeadbeef>.b

    failure_demo.py:114: AssertionError
    _________________________ test_attribute_instance __________________________

        def test_attribute_instance():

            class Foo(object):
                b = 1

    >       assert Foo().b == 2
    E       AssertionError: assert 1 == 2
    E        +  where 1 = <failure_demo.test_attribute_instance.<locals>.Foo object at 0xdeadbeef>.b
    E        +    where <failure_demo.test_attribute_instance.<locals>.Foo object at 0xdeadbeef> = <class 'failure_demo.test_attribute_instance.<locals>.Foo'>()

    failure_demo.py:122: AssertionError
    __________________________ test_attribute_failure __________________________

        def test_attribute_failure():

            class Foo(object):

                def _get_b(self):
                    raise Exception("Failed to get attrib")

                b = property(_get_b)

            i = Foo()
    >       assert i.b == 2

    failure_demo.py:135:
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

    self = <failure_demo.test_attribute_failure.<locals>.Foo object at 0xdeadbeef>

        def _get_b(self):
    >       raise Exception("Failed to get attrib")
    E       Exception: Failed to get attrib

    failure_demo.py:130: Exception
    _________________________ test_attribute_multiple __________________________

        def test_attribute_multiple():

            class Foo(object):
                b = 1

            class Bar(object):
                b = 2

    >       assert Foo().b == Bar().b
    E       AssertionError: assert 1 == 2
    E        +  where 1 = <failure_demo.test_attribute_multiple.<locals>.Foo object at 0xdeadbeef>.b
    E        +    where <failure_demo.test_attribute_multiple.<locals>.Foo object at 0xdeadbeef> = <class 'failure_demo.test_attribute_multiple.<locals>.Foo'>()
    E        +  and   2 = <failure_demo.test_attribute_multiple.<locals>.Bar object at 0xdeadbeef>.b
    E        +    where <failure_demo.test_attribute_multiple.<locals>.Bar object at 0xdeadbeef> = <class 'failure_demo.test_attribute_multiple.<locals>.Bar'>()

    failure_demo.py:146: AssertionError
    __________________________ TestRaises.test_raises __________________________

    self = <failure_demo.TestRaises object at 0xdeadbeef>

        def test_raises(self):
            s = "qwe"  # NOQA
    >       raises(TypeError, "int(s)")

    failure_demo.py:157:
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

    >   int(s)
    E   ValueError: invalid literal for int() with base 10: 'qwe'

    <0-codegen $PYTHON_PREFIX/lib/python3.5/site-packages/_pytest/python_api.py:634>:1: ValueError
    ______________________ TestRaises.test_raises_doesnt _______________________

    self = <failure_demo.TestRaises object at 0xdeadbeef>

        def test_raises_doesnt(self):
    >       raises(IOError, "int('3')")
    E       Failed: DID NOT RAISE <class 'OSError'>

    failure_demo.py:160: Failed
    __________________________ TestRaises.test_raise ___________________________

    self = <failure_demo.TestRaises object at 0xdeadbeef>

        def test_raise(self):
    >       raise ValueError("demo error")
    E       ValueError: demo error

    failure_demo.py:163: ValueError
    ________________________ TestRaises.test_tupleerror ________________________

    self = <failure_demo.TestRaises object at 0xdeadbeef>

        def test_tupleerror(self):
    >       a, b = [1]  # NOQA
    E       ValueError: not enough values to unpack (expected 2, got 1)

    failure_demo.py:166: ValueError
    ______ TestRaises.test_reinterpret_fails_with_print_for_the_fun_of_it ______

    self = <failure_demo.TestRaises object at 0xdeadbeef>

        def test_reinterpret_fails_with_print_for_the_fun_of_it(self):
            items = [1, 2, 3]
            print("items is %r" % items)
    >       a, b = items.pop()
    E       TypeError: 'int' object is not iterable

    failure_demo.py:171: TypeError
    --------------------------- Captured stdout call ---------------------------
    items is [1, 2, 3]
    ________________________ TestRaises.test_some_error ________________________

    self = <failure_demo.TestRaises object at 0xdeadbeef>

        def test_some_error(self):
    >       if namenotexi:  # NOQA
    E       NameError: name 'namenotexi' is not defined

    failure_demo.py:174: NameError
    ____________________ test_dynamic_compile_shows_nicely _____________________

        def test_dynamic_compile_shows_nicely():
            import imp
            import sys

            src = "def foo():\n assert 1 == 0\n"
            name = "abc-123"
            module = imp.new_module(name)
            code = _pytest._code.compile(src, name, "exec")
            py.builtin.exec_(code, module.__dict__)
            sys.modules[name] = module
    >       module.foo()

    failure_demo.py:192:
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

        def foo():
    >    assert 1 == 0
    E    AssertionError

    <2-codegen 'abc-123' $REGENDOC_TMPDIR/assertion/failure_demo.py:189>:2: AssertionError
    ____________________ TestMoreErrors.test_complex_error _____________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_complex_error(self):

            def f():
                return 44

            def g():
                return 43

    >       somefunc(f(), g())

    failure_demo.py:205:
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _
    failure_demo.py:11: in somefunc
        otherfunc(x, y)
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _

    a = 44, b = 43

        def otherfunc(a, b):
    >       assert a == b
    E       assert 44 == 43

    failure_demo.py:7: AssertionError
    ___________________ TestMoreErrors.test_z1_unpack_error ____________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_z1_unpack_error(self):
            items = []
    >       a, b = items
    E       ValueError: not enough values to unpack (expected 2, got 0)

    failure_demo.py:209: ValueError
    ____________________ TestMoreErrors.test_z2_type_error _____________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_z2_type_error(self):
            items = 3
    >       a, b = items
    E       TypeError: 'int' object is not iterable

    failure_demo.py:213: TypeError
    ______________________ TestMoreErrors.test_startswith ______________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_startswith(self):
            s = "123"
            g = "456"
    >       assert s.startswith(g)
    E       AssertionError: assert False
    E        +  where False = <built-in method startswith of str object at 0xdeadbeef>('456')
    E        +    where <built-in method startswith of str object at 0xdeadbeef> = '123'.startswith

    failure_demo.py:218: AssertionError
    __________________ TestMoreErrors.test_startswith_nested ___________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_startswith_nested(self):

            def f():
                return "123"

            def g():
                return "456"

    >       assert f().startswith(g())
    E       AssertionError: assert False
    E        +  where False = <built-in method startswith of str object at 0xdeadbeef>('456')
    E        +    where <built-in method startswith of str object at 0xdeadbeef> = '123'.startswith
    E        +      where '123' = <function TestMoreErrors.test_startswith_nested.<locals>.f at 0xdeadbeef>()
    E        +    and   '456' = <function TestMoreErrors.test_startswith_nested.<locals>.g at 0xdeadbeef>()

    failure_demo.py:228: AssertionError
    _____________________ TestMoreErrors.test_global_func ______________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_global_func(self):
    >       assert isinstance(globf(42), float)
    E       assert False
    E        +  where False = isinstance(43, float)
    E        +    where 43 = globf(42)

    failure_demo.py:231: AssertionError
    _______________________ TestMoreErrors.test_instance _______________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_instance(self):
            self.x = 6 * 7
    >       assert self.x != 42
    E       assert 42 != 42
    E        +  where 42 = <failure_demo.TestMoreErrors object at 0xdeadbeef>.x

    failure_demo.py:235: AssertionError
    _______________________ TestMoreErrors.test_compare ________________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_compare(self):
    >       assert globf(10) < 5
    E       assert 11 < 5
    E        +  where 11 = globf(10)

    failure_demo.py:238: AssertionError
    _____________________ TestMoreErrors.test_try_finally ______________________

    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>

        def test_try_finally(self):
            x = 1
            try:
    >           assert x == 0
    E           assert 1 == 0

    failure_demo.py:243: AssertionError
    ___________________ TestCustomAssertMsg.test_single_line ___________________

    self = <failure_demo.TestCustomAssertMsg object at 0xdeadbeef>

        def test_single_line(self):

            class A(object):
                a = 1

            b = 2
    >       assert A.a == b, "A.a appears not to be b"
    E       AssertionError: A.a appears not to be b
    E       assert 1 == 2
    E        +  where 1 = <class 'failure_demo.TestCustomAssertMsg.test_single_line.<locals>.A'>.a

    failure_demo.py:256: AssertionError
    ____________________ TestCustomAssertMsg.test_multiline ____________________

    self = <failure_demo.TestCustomAssertMsg object at 0xdeadbeef>

        def test_multiline(self):

            class A(object):
                a = 1

            b = 2
    >       assert (
                A.a == b
            ), "A.a appears not to be b\n" "or does not appear to be b\none of those"
    E       AssertionError: A.a appears not to be b
    E         or does not appear to be b
    E         one of those
    E       assert 1 == 2
    E        +  where 1 = <class 'failure_demo.TestCustomAssertMsg.test_multiline.<locals>.A'>.a

    failure_demo.py:264: AssertionError
    ___________________ TestCustomAssertMsg.test_custom_repr ___________________

    self = <failure_demo.TestCustomAssertMsg object at 0xdeadbeef>

        def test_custom_repr(self):

            class JSON(object):
                a = 1

                def __repr__(self):
                    return "This is JSON\n{\n  'foo': 'bar'\n}"

            a = JSON()
            b = 2
    >       assert a.a == b, a
    E       AssertionError: This is JSON
    E         {
    E           'foo': 'bar'
    E         }
    E       assert 1 == 2
    E        +  where 1 = This is JSON\n{\n  'foo': 'bar'\n}.a

    failure_demo.py:278: AssertionError
    ============================= warnings summary =============================
    <undetermined location>
      Metafunc.addcall is deprecated and scheduled to be removed in pytest 4.0.
      Please use Metafunc.parametrize instead.

    -- Docs: http://doc.pytest.org/en/latest/warnings.html
    ================== 42 failed, 1 warnings in 0.12 seconds ===================
