import py
from py.process import cmdexec

def exvalue():
    import sys
    return sys.exc_info()[1]


class Test_exec_cmd:
    def test_simple(self):
        out = cmdexec('echo hallo')
        assert out.strip() == 'hallo'
        assert py.builtin._istext(out)

    def test_simple_newline(self):
        import sys
        out = cmdexec(r"""%s -c "print ('hello')" """ % sys.executable)
        assert out == 'hello\n'
        assert py.builtin._istext(out)

    def test_simple_error(self):
        py.test.raises(cmdexec.Error, cmdexec, 'exit 1')

    def test_simple_error_exact_status(self):
        try:
            cmdexec('exit 1')
        except cmdexec.Error:
            e = exvalue()
            assert e.status == 1
            assert py.builtin._istext(e.out)
            assert py.builtin._istext(e.err)

    def test_err(self):
        try:
            cmdexec('echoqweqwe123 hallo')
            raise AssertionError("command succeeded but shouldn't")
        except cmdexec.Error:
            e = exvalue()
            assert hasattr(e, 'err')
            assert hasattr(e, 'out')
            assert e.err or e.out
