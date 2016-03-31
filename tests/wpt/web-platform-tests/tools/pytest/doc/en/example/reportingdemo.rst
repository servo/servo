
.. _`tbreportdemo`:

Demo of Python failure reports with pytest
==================================================

Here is a nice run of several tens of failures
and how ``pytest`` presents things (unfortunately
not showing the nice colors here in the HTML that you
get on the terminal - we are working on that):

.. code-block:: python

    assertion $ py.test failure_demo.py
    ======= test session starts ========
    platform linux -- Python 3.4.0, pytest-2.9.1, py-1.4.31, pluggy-0.3.1
    rootdir: $REGENDOC_TMPDIR/assertion, inifile: 
    collected 42 items
    
    failure_demo.py FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF
    
    ======= FAILURES ========
    _______ test_generative[0] ________
    
    param1 = 3, param2 = 6
    
        def test_generative(param1, param2):
    >       assert param1 * 2 < param2
    E       assert (3 * 2) < 6
    
    failure_demo.py:16: AssertionError
    _______ TestFailing.test_simple ________
    
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
    
    failure_demo.py:29: AssertionError
    _______ TestFailing.test_simple_multiline ________
    
    self = <failure_demo.TestFailing object at 0xdeadbeef>
    
        def test_simple_multiline(self):
            otherfunc_multi(
                      42,
    >                 6*9)
    
    failure_demo.py:34: 
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ 
    
    a = 42, b = 54
    
        def otherfunc_multi(a,b):
    >       assert (a ==
                    b)
    E       assert 42 == 54
    
    failure_demo.py:12: AssertionError
    _______ TestFailing.test_not ________
    
    self = <failure_demo.TestFailing object at 0xdeadbeef>
    
        def test_not(self):
            def f():
                return 42
    >       assert not f()
    E       assert not 42
    E        +  where 42 = <function TestFailing.test_not.<locals>.f at 0xdeadbeef>()
    
    failure_demo.py:39: AssertionError
    _______ TestSpecialisedExplanations.test_eq_text ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_text(self):
    >       assert 'spam' == 'eggs'
    E       assert 'spam' == 'eggs'
    E         - spam
    E         + eggs
    
    failure_demo.py:43: AssertionError
    _______ TestSpecialisedExplanations.test_eq_similar_text ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_similar_text(self):
    >       assert 'foo 1 bar' == 'foo 2 bar'
    E       assert 'foo 1 bar' == 'foo 2 bar'
    E         - foo 1 bar
    E         ?     ^
    E         + foo 2 bar
    E         ?     ^
    
    failure_demo.py:46: AssertionError
    _______ TestSpecialisedExplanations.test_eq_multiline_text ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_multiline_text(self):
    >       assert 'foo\nspam\nbar' == 'foo\neggs\nbar'
    E       assert 'foo\nspam\nbar' == 'foo\neggs\nbar'
    E           foo
    E         - spam
    E         + eggs
    E           bar
    
    failure_demo.py:49: AssertionError
    _______ TestSpecialisedExplanations.test_eq_long_text ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_long_text(self):
            a = '1'*100 + 'a' + '2'*100
            b = '1'*100 + 'b' + '2'*100
    >       assert a == b
    E       assert '111111111111...2222222222222' == '1111111111111...2222222222222'
    E         Skipping 90 identical leading characters in diff, use -v to show
    E         Skipping 91 identical trailing characters in diff, use -v to show
    E         - 1111111111a222222222
    E         ?           ^
    E         + 1111111111b222222222
    E         ?           ^
    
    failure_demo.py:54: AssertionError
    _______ TestSpecialisedExplanations.test_eq_long_text_multiline ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_long_text_multiline(self):
            a = '1\n'*100 + 'a' + '2\n'*100
            b = '1\n'*100 + 'b' + '2\n'*100
    >       assert a == b
    E       assert '1\n1\n1\n1\n...n2\n2\n2\n2\n' == '1\n1\n1\n1\n1...n2\n2\n2\n2\n'
    E         Skipping 190 identical leading characters in diff, use -v to show
    E         Skipping 191 identical trailing characters in diff, use -v to show
    E           1
    E           1
    E           1
    E           1
    E           1
    E         - a2
    E         + b2
    E           2
    E           2
    E           2
    E           2
    
    failure_demo.py:59: AssertionError
    _______ TestSpecialisedExplanations.test_eq_list ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_list(self):
    >       assert [0, 1, 2] == [0, 1, 3]
    E       assert [0, 1, 2] == [0, 1, 3]
    E         At index 2 diff: 2 != 3
    E         Use -v to get the full diff
    
    failure_demo.py:62: AssertionError
    _______ TestSpecialisedExplanations.test_eq_list_long ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_list_long(self):
            a = [0]*100 + [1] + [3]*100
            b = [0]*100 + [2] + [3]*100
    >       assert a == b
    E       assert [0, 0, 0, 0, 0, 0, ...] == [0, 0, 0, 0, 0, 0, ...]
    E         At index 100 diff: 1 != 2
    E         Use -v to get the full diff
    
    failure_demo.py:67: AssertionError
    _______ TestSpecialisedExplanations.test_eq_dict ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_dict(self):
    >       assert {'a': 0, 'b': 1, 'c': 0} == {'a': 0, 'b': 2, 'd': 0}
    E       assert {'a': 0, 'b': 1, 'c': 0} == {'a': 0, 'b': 2, 'd': 0}
    E         Omitting 1 identical items, use -v to show
    E         Differing items:
    E         {'b': 1} != {'b': 2}
    E         Left contains more items:
    E         {'c': 0}
    E         Right contains more items:
    E         {'d': 0}
    E         Use -v to get the full diff
    
    failure_demo.py:70: AssertionError
    _______ TestSpecialisedExplanations.test_eq_set ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_set(self):
    >       assert set([0, 10, 11, 12]) == set([0, 20, 21])
    E       assert set([0, 10, 11, 12]) == set([0, 20, 21])
    E         Extra items in the left set:
    E         10
    E         11
    E         12
    E         Extra items in the right set:
    E         20
    E         21
    E         Use -v to get the full diff
    
    failure_demo.py:73: AssertionError
    _______ TestSpecialisedExplanations.test_eq_longer_list ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_eq_longer_list(self):
    >       assert [1,2] == [1,2,3]
    E       assert [1, 2] == [1, 2, 3]
    E         Right contains more items, first extra item: 3
    E         Use -v to get the full diff
    
    failure_demo.py:76: AssertionError
    _______ TestSpecialisedExplanations.test_in_list ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_in_list(self):
    >       assert 1 in [0, 2, 3, 4, 5]
    E       assert 1 in [0, 2, 3, 4, 5]
    
    failure_demo.py:79: AssertionError
    _______ TestSpecialisedExplanations.test_not_in_text_multiline ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_not_in_text_multiline(self):
            text = 'some multiline\ntext\nwhich\nincludes foo\nand a\ntail'
    >       assert 'foo' not in text
    E       assert 'foo' not in 'some multiline\ntext\nw...ncludes foo\nand a\ntail'
    E         'foo' is contained here:
    E           some multiline
    E           text
    E           which
    E           includes foo
    E         ?          +++
    E           and a
    E           tail
    
    failure_demo.py:83: AssertionError
    _______ TestSpecialisedExplanations.test_not_in_text_single ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_not_in_text_single(self):
            text = 'single foo line'
    >       assert 'foo' not in text
    E       assert 'foo' not in 'single foo line'
    E         'foo' is contained here:
    E           single foo line
    E         ?        +++
    
    failure_demo.py:87: AssertionError
    _______ TestSpecialisedExplanations.test_not_in_text_single_long ________
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_not_in_text_single_long(self):
            text = 'head ' * 50 + 'foo ' + 'tail ' * 20
    >       assert 'foo' not in text
    E       assert 'foo' not in 'head head head head hea...ail tail tail tail tail '
    E         'foo' is contained here:
    E           head head foo tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail 
    E         ?           +++
    
    failure_demo.py:91: AssertionError
    ______ TestSpecialisedExplanations.test_not_in_text_single_long_term _______
    
    self = <failure_demo.TestSpecialisedExplanations object at 0xdeadbeef>
    
        def test_not_in_text_single_long_term(self):
            text = 'head ' * 50 + 'f'*70 + 'tail ' * 20
    >       assert 'f'*70 not in text
    E       assert 'fffffffffff...ffffffffffff' not in 'head head he...l tail tail '
    E         'ffffffffffffffffff...fffffffffffffffffff' is contained here:
    E           head head fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffftail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail tail 
    E         ?           ++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++
    
    failure_demo.py:95: AssertionError
    _______ test_attribute ________
    
        def test_attribute():
            class Foo(object):
                b = 1
            i = Foo()
    >       assert i.b == 2
    E       assert 1 == 2
    E        +  where 1 = <failure_demo.test_attribute.<locals>.Foo object at 0xdeadbeef>.b
    
    failure_demo.py:102: AssertionError
    _______ test_attribute_instance ________
    
        def test_attribute_instance():
            class Foo(object):
                b = 1
    >       assert Foo().b == 2
    E       assert 1 == 2
    E        +  where 1 = <failure_demo.test_attribute_instance.<locals>.Foo object at 0xdeadbeef>.b
    E        +    where <failure_demo.test_attribute_instance.<locals>.Foo object at 0xdeadbeef> = <class 'failure_demo.test_attribute_instance.<locals>.Foo'>()
    
    failure_demo.py:108: AssertionError
    _______ test_attribute_failure ________
    
        def test_attribute_failure():
            class Foo(object):
                def _get_b(self):
                    raise Exception('Failed to get attrib')
                b = property(_get_b)
            i = Foo()
    >       assert i.b == 2
    
    failure_demo.py:117: 
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ 
    
    self = <failure_demo.test_attribute_failure.<locals>.Foo object at 0xdeadbeef>
    
        def _get_b(self):
    >       raise Exception('Failed to get attrib')
    E       Exception: Failed to get attrib
    
    failure_demo.py:114: Exception
    _______ test_attribute_multiple ________
    
        def test_attribute_multiple():
            class Foo(object):
                b = 1
            class Bar(object):
                b = 2
    >       assert Foo().b == Bar().b
    E       assert 1 == 2
    E        +  where 1 = <failure_demo.test_attribute_multiple.<locals>.Foo object at 0xdeadbeef>.b
    E        +    where <failure_demo.test_attribute_multiple.<locals>.Foo object at 0xdeadbeef> = <class 'failure_demo.test_attribute_multiple.<locals>.Foo'>()
    E        +  and   2 = <failure_demo.test_attribute_multiple.<locals>.Bar object at 0xdeadbeef>.b
    E        +    where <failure_demo.test_attribute_multiple.<locals>.Bar object at 0xdeadbeef> = <class 'failure_demo.test_attribute_multiple.<locals>.Bar'>()
    
    failure_demo.py:125: AssertionError
    _______ TestRaises.test_raises ________
    
    self = <failure_demo.TestRaises object at 0xdeadbeef>
    
        def test_raises(self):
            s = 'qwe'
    >       raises(TypeError, "int(s)")
    
    failure_demo.py:134: 
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ 
    
    >   int(s)
    E   ValueError: invalid literal for int() with base 10: 'qwe'
    
    <0-codegen $PYTHON_PREFIX/lib/python3.4/site-packages/_pytest/python.py:1302>:1: ValueError
    _______ TestRaises.test_raises_doesnt ________
    
    self = <failure_demo.TestRaises object at 0xdeadbeef>
    
        def test_raises_doesnt(self):
    >       raises(IOError, "int('3')")
    E       Failed: DID NOT RAISE <class 'OSError'>
    
    failure_demo.py:137: Failed
    _______ TestRaises.test_raise ________
    
    self = <failure_demo.TestRaises object at 0xdeadbeef>
    
        def test_raise(self):
    >       raise ValueError("demo error")
    E       ValueError: demo error
    
    failure_demo.py:140: ValueError
    _______ TestRaises.test_tupleerror ________
    
    self = <failure_demo.TestRaises object at 0xdeadbeef>
    
        def test_tupleerror(self):
    >       a,b = [1]
    E       ValueError: need more than 1 value to unpack
    
    failure_demo.py:143: ValueError
    ______ TestRaises.test_reinterpret_fails_with_print_for_the_fun_of_it ______
    
    self = <failure_demo.TestRaises object at 0xdeadbeef>
    
        def test_reinterpret_fails_with_print_for_the_fun_of_it(self):
            l = [1,2,3]
            print ("l is %r" % l)
    >       a,b = l.pop()
    E       TypeError: 'int' object is not iterable
    
    failure_demo.py:148: TypeError
    --------------------------- Captured stdout call ---------------------------
    l is [1, 2, 3]
    _______ TestRaises.test_some_error ________
    
    self = <failure_demo.TestRaises object at 0xdeadbeef>
    
        def test_some_error(self):
    >       if namenotexi:
    E       NameError: name 'namenotexi' is not defined
    
    failure_demo.py:151: NameError
    _______ test_dynamic_compile_shows_nicely ________
    
        def test_dynamic_compile_shows_nicely():
            src = 'def foo():\n assert 1 == 0\n'
            name = 'abc-123'
            module = py.std.imp.new_module(name)
            code = _pytest._code.compile(src, name, 'exec')
            py.builtin.exec_(code, module.__dict__)
            py.std.sys.modules[name] = module
    >       module.foo()
    
    failure_demo.py:166: 
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ 
    
        def foo():
    >    assert 1 == 0
    E    assert 1 == 0
    
    <2-codegen 'abc-123' $REGENDOC_TMPDIR/assertion/failure_demo.py:163>:2: AssertionError
    _______ TestMoreErrors.test_complex_error ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_complex_error(self):
            def f():
                return 44
            def g():
                return 43
    >       somefunc(f(), g())
    
    failure_demo.py:176: 
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ 
    failure_demo.py:9: in somefunc
        otherfunc(x,y)
    _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ _ 
    
    a = 44, b = 43
    
        def otherfunc(a,b):
    >       assert a==b
    E       assert 44 == 43
    
    failure_demo.py:6: AssertionError
    _______ TestMoreErrors.test_z1_unpack_error ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_z1_unpack_error(self):
            l = []
    >       a,b  = l
    E       ValueError: need more than 0 values to unpack
    
    failure_demo.py:180: ValueError
    _______ TestMoreErrors.test_z2_type_error ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_z2_type_error(self):
            l = 3
    >       a,b  = l
    E       TypeError: 'int' object is not iterable
    
    failure_demo.py:184: TypeError
    _______ TestMoreErrors.test_startswith ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_startswith(self):
            s = "123"
            g = "456"
    >       assert s.startswith(g)
    E       assert <built-in method startswith of str object at 0xdeadbeef>('456')
    E        +  where <built-in method startswith of str object at 0xdeadbeef> = '123'.startswith
    
    failure_demo.py:189: AssertionError
    _______ TestMoreErrors.test_startswith_nested ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_startswith_nested(self):
            def f():
                return "123"
            def g():
                return "456"
    >       assert f().startswith(g())
    E       assert <built-in method startswith of str object at 0xdeadbeef>('456')
    E        +  where <built-in method startswith of str object at 0xdeadbeef> = '123'.startswith
    E        +    where '123' = <function TestMoreErrors.test_startswith_nested.<locals>.f at 0xdeadbeef>()
    E        +  and   '456' = <function TestMoreErrors.test_startswith_nested.<locals>.g at 0xdeadbeef>()
    
    failure_demo.py:196: AssertionError
    _______ TestMoreErrors.test_global_func ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_global_func(self):
    >       assert isinstance(globf(42), float)
    E       assert isinstance(43, float)
    E        +  where 43 = globf(42)
    
    failure_demo.py:199: AssertionError
    _______ TestMoreErrors.test_instance ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_instance(self):
            self.x = 6*7
    >       assert self.x != 42
    E       assert 42 != 42
    E        +  where 42 = <failure_demo.TestMoreErrors object at 0xdeadbeef>.x
    
    failure_demo.py:203: AssertionError
    _______ TestMoreErrors.test_compare ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_compare(self):
    >       assert globf(10) < 5
    E       assert 11 < 5
    E        +  where 11 = globf(10)
    
    failure_demo.py:206: AssertionError
    _______ TestMoreErrors.test_try_finally ________
    
    self = <failure_demo.TestMoreErrors object at 0xdeadbeef>
    
        def test_try_finally(self):
            x = 1
            try:
    >           assert x == 0
    E           assert 1 == 0
    
    failure_demo.py:211: AssertionError
    _______ TestCustomAssertMsg.test_single_line ________
    
    self = <failure_demo.TestCustomAssertMsg object at 0xdeadbeef>
    
        def test_single_line(self):
            class A:
                a = 1
            b = 2
    >       assert A.a == b, "A.a appears not to be b"
    E       AssertionError: A.a appears not to be b
    E       assert 1 == 2
    E        +  where 1 = <class 'failure_demo.TestCustomAssertMsg.test_single_line.<locals>.A'>.a
    
    failure_demo.py:222: AssertionError
    _______ TestCustomAssertMsg.test_multiline ________
    
    self = <failure_demo.TestCustomAssertMsg object at 0xdeadbeef>
    
        def test_multiline(self):
            class A:
                a = 1
            b = 2
    >       assert A.a == b, "A.a appears not to be b\n" \
                "or does not appear to be b\none of those"
    E       AssertionError: A.a appears not to be b
    E         or does not appear to be b
    E         one of those
    E       assert 1 == 2
    E        +  where 1 = <class 'failure_demo.TestCustomAssertMsg.test_multiline.<locals>.A'>.a
    
    failure_demo.py:228: AssertionError
    _______ TestCustomAssertMsg.test_custom_repr ________
    
    self = <failure_demo.TestCustomAssertMsg object at 0xdeadbeef>
    
        def test_custom_repr(self):
            class JSON:
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
    
    failure_demo.py:238: AssertionError
    ======= 42 failed in 0.12 seconds ========
