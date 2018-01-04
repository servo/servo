
import py

import errno


def test_error_classes():
    for name in errno.errorcode.values():
        x = getattr(py.error, name)
        assert issubclass(x, py.error.Error)
        assert issubclass(x, EnvironmentError)


def test_has_name():
    assert py.error.__name__ == 'py.error'


def test_picklability_issue1():
    import pickle
    e1 = py.error.ENOENT()
    s = pickle.dumps(e1)
    e2 = pickle.loads(s)
    assert isinstance(e2, py.error.ENOENT)


def test_unknown_error():
    num = 3999
    cls = py.error._geterrnoclass(num)
    assert cls.__name__ == 'UnknownErrno%d' % (num,)
    assert issubclass(cls, py.error.Error)
    assert issubclass(cls, EnvironmentError)
    cls2 = py.error._geterrnoclass(num)
    assert cls is cls2


def test_error_conversion_ENOTDIR(testdir):
    p = testdir.makepyfile("")
    excinfo = py.test.raises(py.error.Error, py.error.checked_call, p.listdir)
    assert isinstance(excinfo.value, EnvironmentError)
    assert isinstance(excinfo.value, py.error.Error)
    assert "ENOTDIR" in repr(excinfo.value)


def test_checked_call_supports_kwargs(tmpdir):
    import tempfile
    py.error.checked_call(tempfile.mkdtemp, dir=str(tmpdir))


try:
    import unittest
    unittest.TestCase.assertWarns
except (ImportError, AttributeError):
    pass  # required interface not available
else:
    import sys
    import warnings

    class Case(unittest.TestCase):
        def test_assertWarns(self):
            # Clear everything "py.*" from sys.modules and re-import py
            # as a fresh start
            for mod in tuple(sys.modules.keys()):
                if mod and (mod == 'py' or mod.startswith('py.')):
                    del sys.modules[mod]
            import py

            with self.assertWarns(UserWarning):
                warnings.warn('this should work')
