import sys

import _pytest._code
import py
import pytest


def test_ne():
    code1 = _pytest._code.Code(compile('foo = "bar"', '', 'exec'))
    assert code1 == code1
    code2 = _pytest._code.Code(compile('foo = "baz"', '', 'exec'))
    assert code2 != code1

def test_code_gives_back_name_for_not_existing_file():
    name = 'abc-123'
    co_code = compile("pass\n", name, 'exec')
    assert co_code.co_filename == name
    code = _pytest._code.Code(co_code)
    assert str(code.path) == name
    assert code.fullsource is None

def test_code_with_class():
    class A:
        pass
    pytest.raises(TypeError, "_pytest._code.Code(A)")

if True:
    def x():
        pass

def test_code_fullsource():
    code = _pytest._code.Code(x)
    full = code.fullsource
    assert 'test_code_fullsource()' in str(full)

def test_code_source():
    code = _pytest._code.Code(x)
    src = code.source()
    expected = """def x():
    pass"""
    assert str(src) == expected

def test_frame_getsourcelineno_myself():
    def func():
        return sys._getframe(0)
    f = func()
    f = _pytest._code.Frame(f)
    source, lineno = f.code.fullsource, f.lineno
    assert source[lineno].startswith("        return sys._getframe(0)")

def test_getstatement_empty_fullsource():
    def func():
        return sys._getframe(0)
    f = func()
    f = _pytest._code.Frame(f)
    prop = f.code.__class__.fullsource
    try:
        f.code.__class__.fullsource = None
        assert f.statement == _pytest._code.Source("")
    finally:
        f.code.__class__.fullsource = prop

def test_code_from_func():
    co = _pytest._code.Code(test_frame_getsourcelineno_myself)
    assert co.firstlineno
    assert co.path



def test_builtin_patch_unpatch(monkeypatch):
    cpy_builtin = py.builtin.builtins
    comp = cpy_builtin.compile
    def mycompile(*args, **kwargs):
        return comp(*args, **kwargs)
    class Sub(AssertionError):
        pass
    monkeypatch.setattr(cpy_builtin, 'AssertionError', Sub)
    monkeypatch.setattr(cpy_builtin, 'compile', mycompile)
    _pytest._code.patch_builtins()
    assert cpy_builtin.AssertionError != Sub
    assert cpy_builtin.compile != mycompile
    _pytest._code.unpatch_builtins()
    assert cpy_builtin.AssertionError is Sub
    assert cpy_builtin.compile == mycompile


def test_unicode_handling():
    value = py.builtin._totext('\xc4\x85\xc4\x87\n', 'utf-8').encode('utf8')
    def f():
        raise Exception(value)
    excinfo = pytest.raises(Exception, f)
    str(excinfo)
    if sys.version_info[0] < 3:
        unicode(excinfo)


@pytest.mark.skipif(sys.version_info[0] >= 3, reason='python 2 only issue')
def test_unicode_handling_syntax_error():
    value = py.builtin._totext('\xc4\x85\xc4\x87\n', 'utf-8').encode('utf8')
    def f():
        raise SyntaxError('invalid syntax', (None, 1, 3, value))
    excinfo = pytest.raises(Exception, f)
    str(excinfo)
    if sys.version_info[0] < 3:
        unicode(excinfo)

def test_code_getargs():
    def f1(x):
        pass
    c1 = _pytest._code.Code(f1)
    assert c1.getargs(var=True) == ('x',)

    def f2(x, *y):
        pass
    c2 = _pytest._code.Code(f2)
    assert c2.getargs(var=True) == ('x', 'y')

    def f3(x, **z):
        pass
    c3 = _pytest._code.Code(f3)
    assert c3.getargs(var=True) == ('x', 'z')

    def f4(x, *y, **z):
        pass
    c4 = _pytest._code.Code(f4)
    assert c4.getargs(var=True) == ('x', 'y', 'z')


def test_frame_getargs():
    def f1(x):
        return sys._getframe(0)
    fr1 = _pytest._code.Frame(f1('a'))
    assert fr1.getargs(var=True) == [('x', 'a')]

    def f2(x, *y):
        return sys._getframe(0)
    fr2 = _pytest._code.Frame(f2('a', 'b', 'c'))
    assert fr2.getargs(var=True) == [('x', 'a'), ('y', ('b', 'c'))]

    def f3(x, **z):
        return sys._getframe(0)
    fr3 = _pytest._code.Frame(f3('a', b='c'))
    assert fr3.getargs(var=True) == [('x', 'a'), ('z', {'b': 'c'})]

    def f4(x, *y, **z):
        return sys._getframe(0)
    fr4 = _pytest._code.Frame(f4('a', 'b', c='d'))
    assert fr4.getargs(var=True) == [('x', 'a'), ('y', ('b',)),
                                     ('z', {'c': 'd'})]


class TestExceptionInfo:

    def test_bad_getsource(self):
        try:
            if False: pass
            else: assert False
        except AssertionError:
            exci = _pytest._code.ExceptionInfo()
        assert exci.getrepr()


class TestTracebackEntry:

    def test_getsource(self):
        try:
            if False: pass
            else: assert False
        except AssertionError:
            exci = _pytest._code.ExceptionInfo()
        entry = exci.traceback[0]
        source = entry.getsource()
        assert len(source) == 4
        assert 'else: assert False' in source[3]
