import pytest
import py, sys, os

pytestmark = py.test.mark.skipif("not hasattr(os, 'fork')")


def test_waitfinish_removes_tempdir():
    ff = py.process.ForkedFunc(boxf1)
    assert ff.tempdir.check()
    ff.waitfinish()
    assert not ff.tempdir.check()

def test_tempdir_gets_gc_collected(monkeypatch):
    monkeypatch.setattr(os, 'fork', lambda: os.getpid())
    ff = py.process.ForkedFunc(boxf1)
    assert ff.tempdir.check()
    ff.__del__()
    assert not ff.tempdir.check()

def test_basic_forkedfunc():
    result = py.process.ForkedFunc(boxf1).waitfinish()
    assert result.out == "some out\n"
    assert result.err == "some err\n"
    assert result.exitstatus == 0
    assert result.signal == 0
    assert result.retval == 1

def test_exitstatus():
    def func():
        os._exit(4)
    result = py.process.ForkedFunc(func).waitfinish()
    assert result.exitstatus == 4
    assert result.signal == 0
    assert not result.out
    assert not result.err

def test_execption_in_func():
    def fun():
        raise ValueError(42)
    ff = py.process.ForkedFunc(fun)
    result = ff.waitfinish()
    assert result.exitstatus == ff.EXITSTATUS_EXCEPTION
    assert result.err.find("ValueError: 42") != -1
    assert result.signal == 0
    assert not result.retval

def test_forkedfunc_on_fds():
    result = py.process.ForkedFunc(boxf2).waitfinish()
    assert result.out == "someout"
    assert result.err == "someerr"
    assert result.exitstatus == 0
    assert result.signal == 0
    assert result.retval == 2

def test_forkedfunc_on_fds_output():
    result = py.process.ForkedFunc(boxf3).waitfinish()
    assert result.signal == 11
    assert result.out == "s"


def test_forkedfunc_on_stdout():
    def boxf3():
        import sys
        sys.stdout.write("hello\n")
        os.kill(os.getpid(), 11)
    result = py.process.ForkedFunc(boxf3).waitfinish()
    assert result.signal == 11
    assert result.out == "hello\n"

def test_forkedfunc_signal():
    result = py.process.ForkedFunc(boxseg).waitfinish()
    assert result.retval is None
    if sys.version_info < (2,4):
        py.test.skip("signal detection does not work with python prior 2.4")
    assert result.signal == 11

def test_forkedfunc_huge_data():
    result = py.process.ForkedFunc(boxhuge).waitfinish()
    assert result.out
    assert result.exitstatus == 0
    assert result.signal == 0
    assert result.retval == 3

def test_box_seq():
    # we run many boxes with huge data, just one after another
    for i in range(50):
        result = py.process.ForkedFunc(boxhuge).waitfinish()
        assert result.out
        assert result.exitstatus == 0
        assert result.signal == 0
        assert result.retval == 3

def test_box_in_a_box():
    def boxfun():
        result = py.process.ForkedFunc(boxf2).waitfinish()
        print (result.out)
        sys.stderr.write(result.err + "\n")
        return result.retval

    result = py.process.ForkedFunc(boxfun).waitfinish()
    assert result.out == "someout\n"
    assert result.err == "someerr\n"
    assert result.exitstatus == 0
    assert result.signal == 0
    assert result.retval == 2

def test_kill_func_forked():
    class A:
        pass
    info = A()
    import time

    def box_fun():
        time.sleep(10) # we don't want to last forever here

    ff = py.process.ForkedFunc(box_fun)
    os.kill(ff.pid, 15)
    result = ff.waitfinish()
    if py.std.sys.version_info < (2,4):
        py.test.skip("signal detection does not work with python prior 2.4")
    assert result.signal == 15


def test_hooks(monkeypatch):
    def _boxed():
        return 1

    def _on_start():
        sys.stdout.write("some out\n")
        sys.stdout.flush()

    def _on_exit():
        sys.stderr.write("some err\n")
        sys.stderr.flush()

    result = py.process.ForkedFunc(_boxed, child_on_start=_on_start,
                                   child_on_exit=_on_exit).waitfinish()
    assert result.out == "some out\n"
    assert result.err == "some err\n"
    assert result.exitstatus == 0
    assert result.signal == 0
    assert result.retval == 1


# ======================================================================
# examples
# ======================================================================
#

def boxf1():
    sys.stdout.write("some out\n")
    sys.stderr.write("some err\n")
    return 1

def boxf2():
    os.write(1, "someout".encode('ascii'))
    os.write(2, "someerr".encode('ascii'))
    return 2

def boxf3():
    os.write(1, "s".encode('ascii'))
    os.kill(os.getpid(), 11)

def boxseg():
    os.kill(os.getpid(), 11)

def boxhuge():
    s = " ".encode('ascii')
    os.write(1, s * 10000)
    os.write(2, s * 10000)
    os.write(1, s * 10000)

    os.write(1, s * 10000)
    os.write(2, s * 10000)
    os.write(2, s * 10000)
    os.write(1, s * 10000)
    return 3
