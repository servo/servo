import os
import sys
import textwrap

import pytest
from _pytest.monkeypatch import monkeypatch as MonkeyPatch


def pytest_funcarg__mp(request):
    cwd = os.getcwd()
    sys_path = list(sys.path)

    def cleanup():
        sys.path[:] = sys_path
        os.chdir(cwd)

    request.addfinalizer(cleanup)
    return MonkeyPatch()


def test_setattr():
    class A:
        x = 1

    monkeypatch = MonkeyPatch()
    pytest.raises(AttributeError, "monkeypatch.setattr(A, 'notexists', 2)")
    monkeypatch.setattr(A, 'y', 2, raising=False)
    assert A.y == 2
    monkeypatch.undo()
    assert not hasattr(A, 'y')

    monkeypatch = MonkeyPatch()
    monkeypatch.setattr(A, 'x', 2)
    assert A.x == 2
    monkeypatch.setattr(A, 'x', 3)
    assert A.x == 3
    monkeypatch.undo()
    assert A.x == 1

    A.x = 5
    monkeypatch.undo()  # double-undo makes no modification
    assert A.x == 5


class TestSetattrWithImportPath:
    def test_string_expression(self, monkeypatch):
        monkeypatch.setattr("os.path.abspath", lambda x: "hello2")
        assert os.path.abspath("123") == "hello2"

    def test_string_expression_class(self, monkeypatch):
        monkeypatch.setattr("_pytest.config.Config", 42)
        import _pytest
        assert _pytest.config.Config == 42

    def test_unicode_string(self, monkeypatch):
        monkeypatch.setattr("_pytest.config.Config", 42)
        import _pytest
        assert _pytest.config.Config == 42
        monkeypatch.delattr("_pytest.config.Config")

    def test_wrong_target(self, monkeypatch):
        pytest.raises(TypeError, lambda: monkeypatch.setattr(None, None))

    def test_unknown_import(self, monkeypatch):
        pytest.raises(ImportError,
                      lambda: monkeypatch.setattr("unkn123.classx", None))

    def test_unknown_attr(self, monkeypatch):
        pytest.raises(AttributeError,
                      lambda: monkeypatch.setattr("os.path.qweqwe", None))

    def test_unknown_attr_non_raising(self, monkeypatch):
        # https://github.com/pytest-dev/pytest/issues/746
        monkeypatch.setattr('os.path.qweqwe', 42, raising=False)
        assert os.path.qweqwe == 42

    def test_delattr(self, monkeypatch):
        monkeypatch.delattr("os.path.abspath")
        assert not hasattr(os.path, "abspath")
        monkeypatch.undo()
        assert os.path.abspath


def test_delattr():
    class A:
        x = 1

    monkeypatch = MonkeyPatch()
    monkeypatch.delattr(A, 'x')
    assert not hasattr(A, 'x')
    monkeypatch.undo()
    assert A.x == 1

    monkeypatch = MonkeyPatch()
    monkeypatch.delattr(A, 'x')
    pytest.raises(AttributeError, "monkeypatch.delattr(A, 'y')")
    monkeypatch.delattr(A, 'y', raising=False)
    monkeypatch.setattr(A, 'x', 5, raising=False)
    assert A.x == 5
    monkeypatch.undo()
    assert A.x == 1


def test_setitem():
    d = {'x': 1}
    monkeypatch = MonkeyPatch()
    monkeypatch.setitem(d, 'x', 2)
    monkeypatch.setitem(d, 'y', 1700)
    monkeypatch.setitem(d, 'y', 1700)
    assert d['x'] == 2
    assert d['y'] == 1700
    monkeypatch.setitem(d, 'x', 3)
    assert d['x'] == 3
    monkeypatch.undo()
    assert d['x'] == 1
    assert 'y' not in d
    d['x'] = 5
    monkeypatch.undo()
    assert d['x'] == 5


def test_setitem_deleted_meanwhile():
    d = {}
    monkeypatch = MonkeyPatch()
    monkeypatch.setitem(d, 'x', 2)
    del d['x']
    monkeypatch.undo()
    assert not d


@pytest.mark.parametrize("before", [True, False])
def test_setenv_deleted_meanwhile(before):
    key = "qwpeoip123"
    if before:
        os.environ[key] = "world"
    monkeypatch = MonkeyPatch()
    monkeypatch.setenv(key, 'hello')
    del os.environ[key]
    monkeypatch.undo()
    if before:
        assert os.environ[key] == "world"
        del os.environ[key]
    else:
        assert key not in os.environ


def test_delitem():
    d = {'x': 1}
    monkeypatch = MonkeyPatch()
    monkeypatch.delitem(d, 'x')
    assert 'x' not in d
    monkeypatch.delitem(d, 'y', raising=False)
    pytest.raises(KeyError, "monkeypatch.delitem(d, 'y')")
    assert not d
    monkeypatch.setitem(d, 'y', 1700)
    assert d['y'] == 1700
    d['hello'] = 'world'
    monkeypatch.setitem(d, 'x', 1500)
    assert d['x'] == 1500
    monkeypatch.undo()
    assert d == {'hello': 'world', 'x': 1}


def test_setenv():
    monkeypatch = MonkeyPatch()
    monkeypatch.setenv('XYZ123', 2)
    import os
    assert os.environ['XYZ123'] == "2"
    monkeypatch.undo()
    assert 'XYZ123' not in os.environ


def test_delenv():
    name = 'xyz1234'
    assert name not in os.environ
    monkeypatch = MonkeyPatch()
    pytest.raises(KeyError, "monkeypatch.delenv(%r, raising=True)" % name)
    monkeypatch.delenv(name, raising=False)
    monkeypatch.undo()
    os.environ[name] = "1"
    try:
        monkeypatch = MonkeyPatch()
        monkeypatch.delenv(name)
        assert name not in os.environ
        monkeypatch.setenv(name, "3")
        assert os.environ[name] == "3"
        monkeypatch.undo()
        assert os.environ[name] == "1"
    finally:
        if name in os.environ:
            del os.environ[name]


def test_setenv_prepend():
    import os
    monkeypatch = MonkeyPatch()
    monkeypatch.setenv('XYZ123', 2, prepend="-")
    assert os.environ['XYZ123'] == "2"
    monkeypatch.setenv('XYZ123', 3, prepend="-")
    assert os.environ['XYZ123'] == "3-2"
    monkeypatch.undo()
    assert 'XYZ123' not in os.environ


def test_monkeypatch_plugin(testdir):
    reprec = testdir.inline_runsource("""
        def test_method(monkeypatch):
            assert monkeypatch.__class__.__name__ == "monkeypatch"
    """)
    res = reprec.countoutcomes()
    assert tuple(res) == (1, 0, 0), res


def test_syspath_prepend(mp):
    old = list(sys.path)
    mp.syspath_prepend('world')
    mp.syspath_prepend('hello')
    assert sys.path[0] == "hello"
    assert sys.path[1] == "world"
    mp.undo()
    assert sys.path == old
    mp.undo()
    assert sys.path == old


def test_syspath_prepend_double_undo(mp):
    mp.syspath_prepend('hello world')
    mp.undo()
    sys.path.append('more hello world')
    mp.undo()
    assert sys.path[-1] == 'more hello world'


def test_chdir_with_path_local(mp, tmpdir):
    mp.chdir(tmpdir)
    assert os.getcwd() == tmpdir.strpath


def test_chdir_with_str(mp, tmpdir):
    mp.chdir(tmpdir.strpath)
    assert os.getcwd() == tmpdir.strpath


def test_chdir_undo(mp, tmpdir):
    cwd = os.getcwd()
    mp.chdir(tmpdir)
    mp.undo()
    assert os.getcwd() == cwd


def test_chdir_double_undo(mp, tmpdir):
    mp.chdir(tmpdir.strpath)
    mp.undo()
    tmpdir.chdir()
    mp.undo()
    assert os.getcwd() == tmpdir.strpath


def test_issue185_time_breaks(testdir):
    testdir.makepyfile("""
        import time
        def test_m(monkeypatch):
            def f():
                raise Exception
            monkeypatch.setattr(time, "time", f)
    """)
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("""
        *1 passed*
    """)


def test_importerror(testdir):
    p = testdir.mkpydir("package")
    p.join("a.py").write(textwrap.dedent("""\
        import doesnotexist

        x = 1
    """))
    testdir.tmpdir.join("test_importerror.py").write(textwrap.dedent("""\
        def test_importerror(monkeypatch):
            monkeypatch.setattr('package.a.x', 2)
    """))
    result = testdir.runpytest()
    result.stdout.fnmatch_lines("""
        *import error in package.a: No module named {0}doesnotexist{0}*
    """.format("'" if sys.version_info > (3, 0) else ""))


class SampleNew(object):
    @staticmethod
    def hello():
        return True


class SampleNewInherit(SampleNew):
    pass


class SampleOld:
    # oldstyle on python2
    @staticmethod
    def hello():
        return True


class SampleOldInherit(SampleOld):
    pass


@pytest.mark.parametrize('Sample', [
    SampleNew, SampleNewInherit,
    SampleOld, SampleOldInherit,
], ids=['new', 'new-inherit', 'old', 'old-inherit'])
def test_issue156_undo_staticmethod(Sample):
    monkeypatch = MonkeyPatch()

    monkeypatch.setattr(Sample, 'hello', None)
    assert Sample.hello is None

    monkeypatch.undo()
    assert Sample.hello()

def test_issue1338_name_resolving():
    pytest.importorskip('requests')
    monkeypatch = MonkeyPatch()
    try:
         monkeypatch.delattr('requests.sessions.Session.request')
    finally:
        monkeypatch.undo()