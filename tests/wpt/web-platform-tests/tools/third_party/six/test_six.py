# Copyright (c) 2010-2019 Benjamin Peterson
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.

import operator
import sys
import types
import unittest
import abc

import pytest

import six


def test_add_doc():
    def f():
        """Icky doc"""
        pass
    six._add_doc(f, """New doc""")
    assert f.__doc__ == "New doc"


def test_import_module():
    from logging import handlers
    m = six._import_module("logging.handlers")
    assert m is handlers


def test_integer_types():
    assert isinstance(1, six.integer_types)
    assert isinstance(-1, six.integer_types)
    assert isinstance(six.MAXSIZE + 23, six.integer_types)
    assert not isinstance(.1, six.integer_types)


def test_string_types():
    assert isinstance("hi", six.string_types)
    assert isinstance(six.u("hi"), six.string_types)
    assert issubclass(six.text_type, six.string_types)


def test_class_types():
    class X:
        pass
    class Y(object):
        pass
    assert isinstance(X, six.class_types)
    assert isinstance(Y, six.class_types)
    assert not isinstance(X(), six.class_types)


def test_text_type():
    assert type(six.u("hi")) is six.text_type


def test_binary_type():
    assert type(six.b("hi")) is six.binary_type


def test_MAXSIZE():
    try:
        # This shouldn't raise an overflow error.
        six.MAXSIZE.__index__()
    except AttributeError:
        # Before Python 2.6.
        pass
    pytest.raises(
        (ValueError, OverflowError),
        operator.mul, [None], six.MAXSIZE + 1)


def test_lazy():
    if six.PY3:
        html_name = "html.parser"
    else:
        html_name = "HTMLParser"
    assert html_name not in sys.modules
    mod = six.moves.html_parser
    assert sys.modules[html_name] is mod
    assert "htmlparser" not in six._MovedItems.__dict__


try:
    import _tkinter
except ImportError:
    have_tkinter = False
else:
    have_tkinter = True

have_gdbm = True
try:
    import gdbm
except ImportError:
    try:
        import dbm.gnu
    except ImportError:
        have_gdbm = False

@pytest.mark.parametrize("item_name",
                          [item.name for item in six._moved_attributes])
def test_move_items(item_name):
    """Ensure that everything loads correctly."""
    try:
        item = getattr(six.moves, item_name)
        if isinstance(item, types.ModuleType):
            __import__("six.moves." + item_name)
    except AttributeError:
        if item_name == "zip_longest" and sys.version_info < (2, 6):
            pytest.skip("zip_longest only available on 2.6+")
    except ImportError:
        if item_name == "winreg" and not sys.platform.startswith("win"):
            pytest.skip("Windows only module")
        if item_name.startswith("tkinter"):
            if not have_tkinter:
                pytest.skip("requires tkinter")
            if item_name == "tkinter_ttk" and sys.version_info[:2] <= (2, 6):
                pytest.skip("ttk only available on 2.7+")
        if item_name.startswith("dbm_gnu") and not have_gdbm:
            pytest.skip("requires gdbm")
        raise
    if sys.version_info[:2] >= (2, 6):
        assert item_name in dir(six.moves)


@pytest.mark.parametrize("item_name",
                          [item.name for item in six._urllib_parse_moved_attributes])
def test_move_items_urllib_parse(item_name):
    """Ensure that everything loads correctly."""
    if item_name == "ParseResult" and sys.version_info < (2, 5):
        pytest.skip("ParseResult is only found on 2.5+")
    if item_name in ("parse_qs", "parse_qsl") and sys.version_info < (2, 6):
        pytest.skip("parse_qs[l] is new in 2.6")
    if sys.version_info[:2] >= (2, 6):
        assert item_name in dir(six.moves.urllib.parse)
    getattr(six.moves.urllib.parse, item_name)


@pytest.mark.parametrize("item_name",
                          [item.name for item in six._urllib_error_moved_attributes])
def test_move_items_urllib_error(item_name):
    """Ensure that everything loads correctly."""
    if sys.version_info[:2] >= (2, 6):
        assert item_name in dir(six.moves.urllib.error)
    getattr(six.moves.urllib.error, item_name)


@pytest.mark.parametrize("item_name",
                          [item.name for item in six._urllib_request_moved_attributes])
def test_move_items_urllib_request(item_name):
    """Ensure that everything loads correctly."""
    if sys.version_info[:2] >= (2, 6):
        assert item_name in dir(six.moves.urllib.request)
    getattr(six.moves.urllib.request, item_name)


@pytest.mark.parametrize("item_name",
                          [item.name for item in six._urllib_response_moved_attributes])
def test_move_items_urllib_response(item_name):
    """Ensure that everything loads correctly."""
    if sys.version_info[:2] >= (2, 6):
        assert item_name in dir(six.moves.urllib.response)
    getattr(six.moves.urllib.response, item_name)


@pytest.mark.parametrize("item_name",
                          [item.name for item in six._urllib_robotparser_moved_attributes])
def test_move_items_urllib_robotparser(item_name):
    """Ensure that everything loads correctly."""
    if sys.version_info[:2] >= (2, 6):
        assert item_name in dir(six.moves.urllib.robotparser)
    getattr(six.moves.urllib.robotparser, item_name)


def test_import_moves_error_1():
    from six.moves.urllib.parse import urljoin
    from six import moves
    # In 1.4.1: AttributeError: 'Module_six_moves_urllib_parse' object has no attribute 'urljoin'
    assert moves.urllib.parse.urljoin


def test_import_moves_error_2():
    from six import moves
    assert moves.urllib.parse.urljoin
    # In 1.4.1: ImportError: cannot import name urljoin
    from six.moves.urllib.parse import urljoin


def test_import_moves_error_3():
    from six.moves.urllib.parse import urljoin
    # In 1.4.1: ImportError: cannot import name urljoin
    from six.moves.urllib_parse import urljoin


def test_from_imports():
    from six.moves.queue import Queue
    assert isinstance(Queue, six.class_types)
    from six.moves.configparser import ConfigParser
    assert isinstance(ConfigParser, six.class_types)


def test_filter():
    from six.moves import filter
    f = filter(lambda x: x % 2, range(10))
    assert six.advance_iterator(f) == 1


def test_filter_false():
    from six.moves import filterfalse
    f = filterfalse(lambda x: x % 3, range(10))
    assert six.advance_iterator(f) == 0
    assert six.advance_iterator(f) == 3
    assert six.advance_iterator(f) == 6

def test_map():
    from six.moves import map
    assert six.advance_iterator(map(lambda x: x + 1, range(2))) == 1


def test_getoutput():
    from six.moves import getoutput
    output = getoutput('echo "foo"')
    assert output == 'foo'


def test_zip():
    from six.moves import zip
    assert six.advance_iterator(zip(range(2), range(2))) == (0, 0)


@pytest.mark.skipif("sys.version_info < (2, 6)")
def test_zip_longest():
    from six.moves import zip_longest
    it = zip_longest(range(2), range(1))

    assert six.advance_iterator(it) == (0, 0)
    assert six.advance_iterator(it) == (1, None)


class TestCustomizedMoves:

    def teardown_method(self, meth):
        try:
            del six._MovedItems.spam
        except AttributeError:
            pass
        try:
            del six.moves.__dict__["spam"]
        except KeyError:
            pass


    def test_moved_attribute(self):
        attr = six.MovedAttribute("spam", "foo", "bar")
        if six.PY3:
            assert attr.mod == "bar"
        else:
            assert attr.mod == "foo"
        assert attr.attr == "spam"
        attr = six.MovedAttribute("spam", "foo", "bar", "lemma")
        assert attr.attr == "lemma"
        attr = six.MovedAttribute("spam", "foo", "bar", "lemma", "theorm")
        if six.PY3:
            assert attr.attr == "theorm"
        else:
            assert attr.attr == "lemma"


    def test_moved_module(self):
        attr = six.MovedModule("spam", "foo")
        if six.PY3:
            assert attr.mod == "spam"
        else:
            assert attr.mod == "foo"
        attr = six.MovedModule("spam", "foo", "bar")
        if six.PY3:
            assert attr.mod == "bar"
        else:
            assert attr.mod == "foo"


    def test_custom_move_module(self):
        attr = six.MovedModule("spam", "six", "six")
        six.add_move(attr)
        six.remove_move("spam")
        assert not hasattr(six.moves, "spam")
        attr = six.MovedModule("spam", "six", "six")
        six.add_move(attr)
        from six.moves import spam
        assert spam is six
        six.remove_move("spam")
        assert not hasattr(six.moves, "spam")


    def test_custom_move_attribute(self):
        attr = six.MovedAttribute("spam", "six", "six", "u", "u")
        six.add_move(attr)
        six.remove_move("spam")
        assert not hasattr(six.moves, "spam")
        attr = six.MovedAttribute("spam", "six", "six", "u", "u")
        six.add_move(attr)
        from six.moves import spam
        assert spam is six.u
        six.remove_move("spam")
        assert not hasattr(six.moves, "spam")


    def test_empty_remove(self):
        pytest.raises(AttributeError, six.remove_move, "eggs")


def test_get_unbound_function():
    class X(object):
        def m(self):
            pass
    assert six.get_unbound_function(X.m) is X.__dict__["m"]


def test_get_method_self():
    class X(object):
        def m(self):
            pass
    x = X()
    assert six.get_method_self(x.m) is x
    pytest.raises(AttributeError, six.get_method_self, 42)


def test_get_method_function():
    class X(object):
        def m(self):
            pass
    x = X()
    assert six.get_method_function(x.m) is X.__dict__["m"]
    pytest.raises(AttributeError, six.get_method_function, hasattr)


def test_get_function_closure():
    def f():
        x = 42
        def g():
            return x
        return g
    cell = six.get_function_closure(f())[0]
    assert type(cell).__name__ == "cell"


def test_get_function_code():
    def f():
        pass
    assert isinstance(six.get_function_code(f), types.CodeType)
    if not hasattr(sys, "pypy_version_info"):
        pytest.raises(AttributeError, six.get_function_code, hasattr)


def test_get_function_defaults():
    def f(x, y=3, b=4):
        pass
    assert six.get_function_defaults(f) == (3, 4)


def test_get_function_globals():
    def f():
        pass
    assert six.get_function_globals(f) is globals()


def test_dictionary_iterators(monkeypatch):
    def stock_method_name(iterwhat):
        """Given a method suffix like "lists" or "values", return the name
        of the dict method that delivers those on the version of Python
        we're running in."""
        if six.PY3:
            return iterwhat
        return 'iter' + iterwhat

    class MyDict(dict):
        if not six.PY3:
            def lists(self, **kw):
                return [1, 2, 3]
        def iterlists(self, **kw):
            return iter([1, 2, 3])
    f = MyDict.iterlists
    del MyDict.iterlists
    setattr(MyDict, stock_method_name('lists'), f)

    d = MyDict(zip(range(10), reversed(range(10))))
    for name in "keys", "values", "items", "lists":
        meth = getattr(six, "iter" + name)
        it = meth(d)
        assert not isinstance(it, list)
        assert list(it) == list(getattr(d, name)())
        pytest.raises(StopIteration, six.advance_iterator, it)
        record = []
        def with_kw(*args, **kw):
            record.append(kw["kw"])
            return old(*args)
        old = getattr(MyDict, stock_method_name(name))
        monkeypatch.setattr(MyDict, stock_method_name(name), with_kw)
        meth(d, kw=42)
        assert record == [42]
        monkeypatch.undo()


@pytest.mark.skipif("sys.version_info[:2] < (2, 7)",
                reason="view methods on dictionaries only available on 2.7+")
def test_dictionary_views():
    d = dict(zip(range(10), (range(11, 20))))
    for name in "keys", "values", "items":
        meth = getattr(six, "view" + name)
        view = meth(d)
        assert set(view) == set(getattr(d, name)())


def test_advance_iterator():
    assert six.next is six.advance_iterator
    l = [1, 2]
    it = iter(l)
    assert six.next(it) == 1
    assert six.next(it) == 2
    pytest.raises(StopIteration, six.next, it)
    pytest.raises(StopIteration, six.next, it)


def test_iterator():
    class myiter(six.Iterator):
        def __next__(self):
            return 13
    assert six.advance_iterator(myiter()) == 13
    class myitersub(myiter):
        def __next__(self):
            return 14
    assert six.advance_iterator(myitersub()) == 14


def test_callable():
    class X:
        def __call__(self):
            pass
        def method(self):
            pass
    assert six.callable(X)
    assert six.callable(X())
    assert six.callable(test_callable)
    assert six.callable(hasattr)
    assert six.callable(X.method)
    assert six.callable(X().method)
    assert not six.callable(4)
    assert not six.callable("string")


def test_create_bound_method():
    class X(object):
        pass
    def f(self):
        return self
    x = X()
    b = six.create_bound_method(f, x)
    assert isinstance(b, types.MethodType)
    assert b() is x


def test_create_unbound_method():
    class X(object):
        pass

    def f(self):
        return self
    u = six.create_unbound_method(f, X)
    pytest.raises(TypeError, u)
    if six.PY2:
        assert isinstance(u, types.MethodType)
    x = X()
    assert f(x) is x


if six.PY3:

    def test_b():
        data = six.b("\xff")
        assert isinstance(data, bytes)
        assert len(data) == 1
        assert data == bytes([255])


    def test_u():
        s = six.u("hi \u0439 \U00000439 \\ \\\\ \n")
        assert isinstance(s, str)
        assert s == "hi \u0439 \U00000439 \\ \\\\ \n"

else:

    def test_b():
        data = six.b("\xff")
        assert isinstance(data, str)
        assert len(data) == 1
        assert data == "\xff"


    def test_u():
        s = six.u("hi \u0439 \U00000439 \\ \\\\ \n")
        assert isinstance(s, unicode)
        assert s == "hi \xd0\xb9 \xd0\xb9 \\ \\\\ \n".decode("utf8")


def test_u_escapes():
    s = six.u("\u1234")
    assert len(s) == 1


def test_unichr():
    assert six.u("\u1234") == six.unichr(0x1234)
    assert type(six.u("\u1234")) is type(six.unichr(0x1234))


def test_int2byte():
    assert six.int2byte(3) == six.b("\x03")
    pytest.raises(Exception, six.int2byte, 256)


def test_byte2int():
    assert six.byte2int(six.b("\x03")) == 3
    assert six.byte2int(six.b("\x03\x04")) == 3
    pytest.raises(IndexError, six.byte2int, six.b(""))


def test_bytesindex():
    assert six.indexbytes(six.b("hello"), 3) == ord("l")


def test_bytesiter():
    it = six.iterbytes(six.b("hi"))
    assert six.next(it) == ord("h")
    assert six.next(it) == ord("i")
    pytest.raises(StopIteration, six.next, it)


def test_StringIO():
    fp = six.StringIO()
    fp.write(six.u("hello"))
    assert fp.getvalue() == six.u("hello")


def test_BytesIO():
    fp = six.BytesIO()
    fp.write(six.b("hello"))
    assert fp.getvalue() == six.b("hello")


def test_exec_():
    def f():
        l = []
        six.exec_("l.append(1)")
        assert l == [1]
    f()
    ns = {}
    six.exec_("x = 42", ns)
    assert ns["x"] == 42
    glob = {}
    loc = {}
    six.exec_("global y; y = 42; x = 12", glob, loc)
    assert glob["y"] == 42
    assert "x" not in glob
    assert loc["x"] == 12
    assert "y" not in loc


def test_reraise():
    def get_next(tb):
        if six.PY3:
            return tb.tb_next.tb_next
        else:
            return tb.tb_next
    e = Exception("blah")
    try:
        raise e
    except Exception:
        tp, val, tb = sys.exc_info()
    try:
        six.reraise(tp, val, tb)
    except Exception:
        tp2, value2, tb2 = sys.exc_info()
        assert tp2 is Exception
        assert value2 is e
        assert tb is get_next(tb2)
    try:
        six.reraise(tp, val)
    except Exception:
        tp2, value2, tb2 = sys.exc_info()
        assert tp2 is Exception
        assert value2 is e
        assert tb2 is not tb
    try:
        six.reraise(tp, val, tb2)
    except Exception:
        tp2, value2, tb3 = sys.exc_info()
        assert tp2 is Exception
        assert value2 is e
        assert get_next(tb3) is tb2
    try:
        six.reraise(tp, None, tb)
    except Exception:
        tp2, value2, tb2 = sys.exc_info()
        assert tp2 is Exception
        assert value2 is not val
        assert isinstance(value2, Exception)
        assert tb is get_next(tb2)


def test_raise_from():
    try:
        try:
            raise Exception("blah")
        except Exception:
            ctx = sys.exc_info()[1]
            f = Exception("foo")
            six.raise_from(f, None)
    except Exception:
        tp, val, tb = sys.exc_info()
    if sys.version_info[:2] > (3, 0):
        # We should have done a raise f from None equivalent.
        assert val.__cause__ is None
        assert val.__context__ is ctx
    if sys.version_info[:2] >= (3, 3):
        # And that should suppress the context on the exception.
        assert val.__suppress_context__
    # For all versions the outer exception should have raised successfully.
    assert str(val) == "foo"


def test_print_():
    save = sys.stdout
    out = sys.stdout = six.moves.StringIO()
    try:
        six.print_("Hello,", "person!")
    finally:
        sys.stdout = save
    assert out.getvalue() == "Hello, person!\n"
    out = six.StringIO()
    six.print_("Hello,", "person!", file=out)
    assert out.getvalue() == "Hello, person!\n"
    out = six.StringIO()
    six.print_("Hello,", "person!", file=out, end="")
    assert out.getvalue() == "Hello, person!"
    out = six.StringIO()
    six.print_("Hello,", "person!", file=out, sep="X")
    assert out.getvalue() == "Hello,Xperson!\n"
    out = six.StringIO()
    six.print_(six.u("Hello,"), six.u("person!"), file=out)
    result = out.getvalue()
    assert isinstance(result, six.text_type)
    assert result == six.u("Hello, person!\n")
    six.print_("Hello", file=None) # This works.
    out = six.StringIO()
    six.print_(None, file=out)
    assert out.getvalue() == "None\n"
    class FlushableStringIO(six.StringIO):
        def __init__(self):
            six.StringIO.__init__(self)
            self.flushed = False
        def flush(self):
            self.flushed = True
    out = FlushableStringIO()
    six.print_("Hello", file=out)
    assert not out.flushed
    six.print_("Hello", file=out, flush=True)
    assert out.flushed


@pytest.mark.skipif("sys.version_info[:2] >= (2, 6)")
def test_print_encoding(monkeypatch):
    # Fool the type checking in print_.
    monkeypatch.setattr(six, "file", six.BytesIO, raising=False)
    out = six.BytesIO()
    out.encoding = "utf-8"
    out.errors = None
    six.print_(six.u("\u053c"), end="", file=out)
    assert out.getvalue() == six.b("\xd4\xbc")
    out = six.BytesIO()
    out.encoding = "ascii"
    out.errors = "strict"
    pytest.raises(UnicodeEncodeError, six.print_, six.u("\u053c"), file=out)
    out.errors = "backslashreplace"
    six.print_(six.u("\u053c"), end="", file=out)
    assert out.getvalue() == six.b("\\u053c")


def test_print_exceptions():
    pytest.raises(TypeError, six.print_, x=3)
    pytest.raises(TypeError, six.print_, end=3)
    pytest.raises(TypeError, six.print_, sep=42)


def test_with_metaclass():
    class Meta(type):
        pass
    class X(six.with_metaclass(Meta)):
        pass
    assert type(X) is Meta
    assert issubclass(X, object)
    class Base(object):
        pass
    class X(six.with_metaclass(Meta, Base)):
        pass
    assert type(X) is Meta
    assert issubclass(X, Base)
    class Base2(object):
        pass
    class X(six.with_metaclass(Meta, Base, Base2)):
        pass
    assert type(X) is Meta
    assert issubclass(X, Base)
    assert issubclass(X, Base2)
    assert X.__mro__ == (X, Base, Base2, object)
    class X(six.with_metaclass(Meta)):
        pass
    class MetaSub(Meta):
        pass
    class Y(six.with_metaclass(MetaSub, X)):
        pass
    assert type(Y) is MetaSub
    assert Y.__mro__ == (Y, X, object)


@pytest.mark.skipif("sys.version_info[:2] < (2, 7)")
def test_with_metaclass_typing():
    try:
        import typing
    except ImportError:
        pytest.skip("typing module required")
    class Meta(type):
        pass
    if sys.version_info[:2] < (3, 7):
        # Generics with custom metaclasses were broken on older versions.
        class Meta(Meta, typing.GenericMeta):
            pass
    T = typing.TypeVar('T')
    class G(six.with_metaclass(Meta, typing.Generic[T])):
        pass
    class GA(six.with_metaclass(abc.ABCMeta, typing.Generic[T])):
        pass
    assert isinstance(G, Meta)
    assert isinstance(GA, abc.ABCMeta)
    assert G[int] is not G[G[int]]
    assert GA[int] is not GA[GA[int]]
    assert G.__bases__ == (typing.Generic,)
    assert G.__orig_bases__ == (typing.Generic[T],)


@pytest.mark.skipif("sys.version_info[:2] < (3, 7)")
def test_with_metaclass_pep_560():
    class Meta(type):
        pass
    class A:
        pass
    class B:
        pass
    class Fake:
        def __mro_entries__(self, bases):
            return (A, B)
    fake = Fake()
    class G(six.with_metaclass(Meta, fake)):
        pass
    class GA(six.with_metaclass(abc.ABCMeta, fake)):
        pass
    assert isinstance(G, Meta)
    assert isinstance(GA, abc.ABCMeta)
    assert G.__bases__ == (A, B)
    assert G.__orig_bases__ == (fake,)


@pytest.mark.skipif("sys.version_info[:2] < (3, 0)")
def test_with_metaclass_prepare():
    """Test that with_metaclass causes Meta.__prepare__ to be called with the correct arguments."""

    class MyDict(dict):
        pass

    class Meta(type):

        @classmethod
        def __prepare__(cls, name, bases):
            namespace = MyDict(super().__prepare__(name, bases), cls=cls, bases=bases)
            namespace['namespace'] = namespace
            return namespace

    class Base(object):
        pass

    bases = (Base,)

    class X(six.with_metaclass(Meta, *bases)):
        pass

    assert getattr(X, 'cls', type) is Meta
    assert getattr(X, 'bases', ()) == bases
    assert isinstance(getattr(X, 'namespace', {}), MyDict)


def test_wraps():
    def f(g):
        @six.wraps(g)
        def w():
            return 42
        return w
    def k():
        pass
    original_k = k
    k = f(f(k))
    assert hasattr(k, '__wrapped__')
    k = k.__wrapped__
    assert hasattr(k, '__wrapped__')
    k = k.__wrapped__
    assert k is original_k
    assert not hasattr(k, '__wrapped__')

    def f(g, assign, update):
        def w():
            return 42
        w.glue = {"foo" : "bar"}
        return six.wraps(g, assign, update)(w)
    k.glue = {"melon" : "egg"}
    k.turnip = 43
    k = f(k, ["turnip"], ["glue"])
    assert k.__name__ == "w"
    assert k.turnip == 43
    assert k.glue == {"melon" : "egg", "foo" : "bar"}


def test_add_metaclass():
    class Meta(type):
        pass
    class X:
        "success"
    X = six.add_metaclass(Meta)(X)
    assert type(X) is Meta
    assert issubclass(X, object)
    assert X.__module__ == __name__
    assert X.__doc__ == "success"
    class Base(object):
        pass
    class X(Base):
        pass
    X = six.add_metaclass(Meta)(X)
    assert type(X) is Meta
    assert issubclass(X, Base)
    class Base2(object):
        pass
    class X(Base, Base2):
        pass
    X = six.add_metaclass(Meta)(X)
    assert type(X) is Meta
    assert issubclass(X, Base)
    assert issubclass(X, Base2)

    # Test a second-generation subclass of a type.
    class Meta1(type):
        m1 = "m1"
    class Meta2(Meta1):
        m2 = "m2"
    class Base:
        b = "b"
    Base = six.add_metaclass(Meta1)(Base)
    class X(Base):
        x = "x"
    X = six.add_metaclass(Meta2)(X)
    assert type(X) is Meta2
    assert issubclass(X, Base)
    assert type(Base) is Meta1
    assert "__dict__" not in vars(X)
    instance = X()
    instance.attr = "test"
    assert vars(instance) == {"attr": "test"}
    assert instance.b == Base.b
    assert instance.x == X.x

    # Test a class with slots.
    class MySlots(object):
        __slots__ = ["a", "b"]
    MySlots = six.add_metaclass(Meta1)(MySlots)

    assert MySlots.__slots__ == ["a", "b"]
    instance = MySlots()
    instance.a = "foo"
    pytest.raises(AttributeError, setattr, instance, "c", "baz")

    # Test a class with string for slots.
    class MyStringSlots(object):
        __slots__ = "ab"
    MyStringSlots = six.add_metaclass(Meta1)(MyStringSlots)
    assert MyStringSlots.__slots__ == "ab"
    instance = MyStringSlots()
    instance.ab = "foo"
    pytest.raises(AttributeError, setattr, instance, "a", "baz")
    pytest.raises(AttributeError, setattr, instance, "b", "baz")

    class MySlotsWeakref(object):
        __slots__ = "__weakref__",
    MySlotsWeakref = six.add_metaclass(Meta)(MySlotsWeakref)
    assert type(MySlotsWeakref) is Meta


@pytest.mark.skipif("sys.version_info[:2] < (3, 3)")
def test_add_metaclass_nested():
    # Regression test for https://github.com/benjaminp/six/issues/259
    class Meta(type):
        pass

    class A:
        class B: pass

    expected = 'test_add_metaclass_nested.<locals>.A.B'

    assert A.B.__qualname__ == expected

    class A:
        @six.add_metaclass(Meta)
        class B: pass

    assert A.B.__qualname__ == expected


@pytest.mark.skipif("sys.version_info[:2] < (2, 7) or sys.version_info[:2] in ((3, 0), (3, 1))")
def test_assertCountEqual():
    class TestAssertCountEqual(unittest.TestCase):
        def test(self):
            with self.assertRaises(AssertionError):
                six.assertCountEqual(self, (1, 2), [3, 4, 5])

            six.assertCountEqual(self, (1, 2), [2, 1])

    TestAssertCountEqual('test').test()


@pytest.mark.skipif("sys.version_info[:2] < (2, 7)")
def test_assertRegex():
    class TestAssertRegex(unittest.TestCase):
        def test(self):
            with self.assertRaises(AssertionError):
                six.assertRegex(self, 'test', r'^a')

            six.assertRegex(self, 'test', r'^t')

    TestAssertRegex('test').test()


@pytest.mark.skipif("sys.version_info[:2] < (2, 7)")
def test_assertRaisesRegex():
    class TestAssertRaisesRegex(unittest.TestCase):
        def test(self):
            with six.assertRaisesRegex(self, AssertionError, '^Foo'):
                raise AssertionError('Foo')

            with self.assertRaises(AssertionError):
                with six.assertRaisesRegex(self, AssertionError, r'^Foo'):
                    raise AssertionError('Bar')

    TestAssertRaisesRegex('test').test()


def test_python_2_unicode_compatible():
    @six.python_2_unicode_compatible
    class MyTest(object):
        def __str__(self):
            return six.u('hello')

        def __bytes__(self):
            return six.b('hello')

    my_test = MyTest()

    if six.PY2:
        assert str(my_test) == six.b("hello")
        assert unicode(my_test) == six.u("hello")
    elif six.PY3:
        assert bytes(my_test) == six.b("hello")
        assert str(my_test) == six.u("hello")

    assert getattr(six.moves.builtins, 'bytes', str)(my_test) == six.b("hello")


class EnsureTests:

    # grinning face emoji
    UNICODE_EMOJI = six.u("\U0001F600")
    BINARY_EMOJI = b"\xf0\x9f\x98\x80"

    def test_ensure_binary_raise_type_error(self):
        with pytest.raises(TypeError):
            six.ensure_str(8)

    def test_errors_and_encoding(self):
        six.ensure_binary(self.UNICODE_EMOJI, encoding='latin-1', errors='ignore')
        with pytest.raises(UnicodeEncodeError):
            six.ensure_binary(self.UNICODE_EMOJI, encoding='latin-1', errors='strict')

    def test_ensure_binary_raise(self):
        converted_unicode = six.ensure_binary(self.UNICODE_EMOJI, encoding='utf-8', errors='strict')
        converted_binary = six.ensure_binary(self.BINARY_EMOJI, encoding="utf-8", errors='strict')
        if six.PY2:
            # PY2: unicode -> str
            assert converted_unicode == self.BINARY_EMOJI and isinstance(converted_unicode, str)
            # PY2: str -> str
            assert converted_binary == self.BINARY_EMOJI and isinstance(converted_binary, str)
        else:
            # PY3: str -> bytes
            assert converted_unicode == self.BINARY_EMOJI and isinstance(converted_unicode, bytes)
            # PY3: bytes -> bytes
            assert converted_binary == self.BINARY_EMOJI and isinstance(converted_binary, bytes)

    def test_ensure_str(self):
        converted_unicode = six.ensure_str(self.UNICODE_EMOJI, encoding='utf-8', errors='strict')
        converted_binary = six.ensure_str(self.BINARY_EMOJI, encoding="utf-8", errors='strict')
        if six.PY2:
            # PY2: unicode -> str
            assert converted_unicode == self.BINARY_EMOJI and isinstance(converted_unicode, str)
            # PY2: str -> str
            assert converted_binary == self.BINARY_EMOJI and isinstance(converted_binary, str)
        else:
            # PY3: str -> str
            assert converted_unicode == self.UNICODE_EMOJI and isinstance(converted_unicode, str)
            # PY3: bytes -> str
            assert converted_binary == self.UNICODE_EMOJI and isinstance(converted_unicode, str)

    def test_ensure_text(self):
        converted_unicode = six.ensure_text(self.UNICODE_EMOJI, encoding='utf-8', errors='strict')
        converted_binary = six.ensure_text(self.BINARY_EMOJI, encoding="utf-8", errors='strict')
        if six.PY2:
            # PY2: unicode -> unicode
            assert converted_unicode == self.UNICODE_EMOJI and isinstance(converted_unicode, unicode)
            # PY2: str -> unicode
            assert converted_binary == self.UNICODE_EMOJI and isinstance(converted_unicode, unicode)
        else:
            # PY3: str -> str
            assert converted_unicode == self.UNICODE_EMOJI and isinstance(converted_unicode, str)
            # PY3: bytes -> str
            assert converted_binary == self.UNICODE_EMOJI and isinstance(converted_unicode, str)
