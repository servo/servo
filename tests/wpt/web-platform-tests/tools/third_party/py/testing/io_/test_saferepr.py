# -*- coding: utf-8 -*-

from __future__ import generators
import py
import sys

saferepr = py.io.saferepr

class TestSafeRepr:
    def test_simple_repr(self):
        assert saferepr(1) == '1'
        assert saferepr(None) == 'None'

    def test_maxsize(self):
        s = saferepr('x'*50, maxsize=25)
        assert len(s) == 25
        expected = repr('x'*10 + '...' + 'x'*10)
        assert s == expected

    def test_maxsize_error_on_instance(self):
        class A:
            def __repr__(self):
                raise ValueError('...')

        s = saferepr(('*'*50, A()), maxsize=25)
        assert len(s) == 25
        assert s[0] == '(' and s[-1] == ')'

    def test_exceptions(self):
        class BrokenRepr:
            def __init__(self, ex):
                self.ex = ex
                foo = 0
            def __repr__(self):
                raise self.ex
        class BrokenReprException(Exception):
            __str__ = None
            __repr__ = None
        assert 'Exception' in saferepr(BrokenRepr(Exception("broken")))
        s = saferepr(BrokenReprException("really broken"))
        assert 'TypeError' in s
        assert 'TypeError' in saferepr(BrokenRepr("string"))

        s2 = saferepr(BrokenRepr(BrokenReprException('omg even worse')))
        assert 'NameError' not in s2
        assert 'unknown' in s2

    def test_big_repr(self):
        from py._io.saferepr import SafeRepr
        assert len(saferepr(range(1000))) <= \
               len('[' + SafeRepr().maxlist * "1000" + ']')

    def test_repr_on_newstyle(self):
        class Function(object):
            def __repr__(self):
                return "<%s>" %(self.name)
        try:
            s = saferepr(Function())
        except Exception:
            py.test.fail("saferepr failed for newstyle class")

    def test_unicode(self):
        val = py.builtin._totext('£€', 'utf-8')
        reprval = py.builtin._totext("'£€'", 'utf-8')
        assert saferepr(val) == reprval

def test_unicode_handling():
    value = py.builtin._totext('\xc4\x85\xc4\x87\n', 'utf-8').encode('utf8')
    def f():
        raise Exception(value)
    excinfo = py.test.raises(Exception, f)
    s = str(excinfo)
    if sys.version_info[0] < 3:
        u = unicode(excinfo)

