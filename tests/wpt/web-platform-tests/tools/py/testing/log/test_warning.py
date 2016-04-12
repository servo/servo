import pytest
import py

mypath = py.path.local(__file__).new(ext=".py")

@pytest.mark.xfail
def test_forwarding_to_warnings_module():
    pytest.deprecated_call(py.log._apiwarn, "1.3", "..")

def test_apiwarn_functional(recwarn):
    capture = py.io.StdCapture()
    py.log._apiwarn("x.y.z", "something", stacklevel=1)
    out, err = capture.reset()
    py.builtin.print_("out", out)
    py.builtin.print_("err", err)
    assert err.find("x.y.z") != -1
    lno = py.code.getrawcode(test_apiwarn_functional).co_firstlineno + 2
    exp = "%s:%s" % (mypath, lno)
    assert err.find(exp) != -1

def test_stacklevel(recwarn):
    def f():
        py.log._apiwarn("x", "some", stacklevel=2)
    # 3
    # 4
    capture = py.io.StdCapture()
    f()
    out, err = capture.reset()
    lno = py.code.getrawcode(test_stacklevel).co_firstlineno + 6
    warning = str(err)
    assert warning.find(":%s" % lno) != -1

def test_stacklevel_initpkg_with_resolve(testdir, recwarn):
    testdir.makepyfile(modabc="""
        import py
        def f():
            py.log._apiwarn("x", "some", stacklevel="apipkg123")
    """)
    testdir.makepyfile(apipkg123="""
        def __getattr__():
            import modabc
            modabc.f()
    """)
    p = testdir.makepyfile("""
        import apipkg123
        apipkg123.__getattr__()
    """)
    capture = py.io.StdCapture()
    p.pyimport()
    out, err = capture.reset()
    warning = str(err)
    loc = 'test_stacklevel_initpkg_with_resolve.py:2'
    assert warning.find(loc) != -1

def test_stacklevel_initpkg_no_resolve(recwarn):
    def f():
        py.log._apiwarn("x", "some", stacklevel="apipkg")
    capture = py.io.StdCapture()
    f()
    out, err = capture.reset()
    lno = py.code.getrawcode(test_stacklevel_initpkg_no_resolve).co_firstlineno + 2
    warning = str(err)
    assert warning.find(":%s" % lno) != -1


def test_function(recwarn):
    capture = py.io.StdCapture()
    py.log._apiwarn("x.y.z", "something", function=test_function)
    out, err = capture.reset()
    py.builtin.print_("out", out)
    py.builtin.print_("err", err)
    assert err.find("x.y.z") != -1
    lno = py.code.getrawcode(test_function).co_firstlineno
    exp = "%s:%s" % (mypath, lno)
    assert err.find(exp) != -1

