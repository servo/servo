from __future__ import absolute_import, division, print_function
import sys
import py
import pytest

from _pytest.tmpdir import tmpdir


def test_funcarg(testdir):
    testdir.makepyfile("""
            def pytest_generate_tests(metafunc):
                metafunc.addcall(id='a')
                metafunc.addcall(id='b')
            def test_func(tmpdir): pass
    """)
    from _pytest.tmpdir import TempdirFactory
    reprec = testdir.inline_run()
    calls = reprec.getcalls("pytest_runtest_setup")
    item = calls[0].item
    config = item.config
    tmpdirhandler = TempdirFactory(config)
    item._initrequest()
    p = tmpdir(item._request, tmpdirhandler)
    assert p.check()
    bn = p.basename.strip("0123456789")
    assert bn.endswith("test_func_a_")
    item.name = "qwe/\\abc"
    p = tmpdir(item._request, tmpdirhandler)
    assert p.check()
    bn = p.basename.strip("0123456789")
    assert bn == "qwe__abc"


def test_ensuretemp(recwarn):
    d1 = pytest.ensuretemp('hello')
    d2 = pytest.ensuretemp('hello')
    assert d1 == d2
    assert d1.check(dir=1)


class TestTempdirHandler(object):
    def test_mktemp(self, testdir):
        from _pytest.tmpdir import TempdirFactory
        config = testdir.parseconfig()
        config.option.basetemp = testdir.mkdir("hello")
        t = TempdirFactory(config)
        tmp = t.mktemp("world")
        assert tmp.relto(t.getbasetemp()) == "world0"
        tmp = t.mktemp("this")
        assert tmp.relto(t.getbasetemp()).startswith("this")
        tmp2 = t.mktemp("this")
        assert tmp2.relto(t.getbasetemp()).startswith("this")
        assert tmp2 != tmp


class TestConfigTmpdir(object):
    def test_getbasetemp_custom_removes_old(self, testdir):
        mytemp = testdir.tmpdir.join("xyz")
        p = testdir.makepyfile("""
            def test_1(tmpdir):
                pass
        """)
        testdir.runpytest(p, '--basetemp=%s' % mytemp)
        mytemp.check()
        mytemp.ensure("hello")

        testdir.runpytest(p, '--basetemp=%s' % mytemp)
        mytemp.check()
        assert not mytemp.join("hello").check()


def test_basetemp(testdir):
    mytemp = testdir.tmpdir.mkdir("mytemp")
    p = testdir.makepyfile("""
        import pytest
        def test_1():
            pytest.ensuretemp("hello")
    """)
    result = testdir.runpytest(p, '--basetemp=%s' % mytemp)
    assert result.ret == 0
    assert mytemp.join('hello').check()


@pytest.mark.skipif(not hasattr(py.path.local, 'mksymlinkto'),
                    reason="symlink not available on this platform")
def test_tmpdir_always_is_realpath(testdir):
    # the reason why tmpdir should be a realpath is that
    # when you cd to it and do "os.getcwd()" you will anyway
    # get the realpath.  Using the symlinked path can thus
    # easily result in path-inequality
    # XXX if that proves to be a problem, consider using
    # os.environ["PWD"]
    realtemp = testdir.tmpdir.mkdir("myrealtemp")
    linktemp = testdir.tmpdir.join("symlinktemp")
    linktemp.mksymlinkto(realtemp)
    p = testdir.makepyfile("""
        def test_1(tmpdir):
            import os
            assert os.path.realpath(str(tmpdir)) == str(tmpdir)
    """)
    result = testdir.runpytest("-s", p, '--basetemp=%s/bt' % linktemp)
    assert not result.ret


def test_tmpdir_too_long_on_parametrization(testdir):
    testdir.makepyfile("""
        import pytest
        @pytest.mark.parametrize("arg", ["1"*1000])
        def test_some(arg, tmpdir):
            tmpdir.ensure("hello")
    """)
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmpdir_factory(testdir):
    testdir.makepyfile("""
        import pytest
        @pytest.fixture(scope='session')
        def session_dir(tmpdir_factory):
            return tmpdir_factory.mktemp('data', numbered=False)
        def test_some(session_dir):
            session_dir.isdir()
    """)
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


def test_tmpdir_fallback_tox_env(testdir, monkeypatch):
    """Test that tmpdir works even if environment variables required by getpass
    module are missing (#1010).
    """
    monkeypatch.delenv('USER', raising=False)
    monkeypatch.delenv('USERNAME', raising=False)
    testdir.makepyfile("""
        import pytest
        def test_some(tmpdir):
            assert tmpdir.isdir()
    """)
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.fixture
def break_getuser(monkeypatch):
    monkeypatch.setattr('os.getuid', lambda: -1)
    # taken from python 2.7/3.4
    for envvar in ('LOGNAME', 'USER', 'LNAME', 'USERNAME'):
        monkeypatch.delenv(envvar, raising=False)


@pytest.mark.usefixtures("break_getuser")
@pytest.mark.skipif(sys.platform.startswith('win'), reason='no os.getuid on windows')
def test_tmpdir_fallback_uid_not_found(testdir):
    """Test that tmpdir works even if the current process's user id does not
    correspond to a valid user.
    """

    testdir.makepyfile("""
        import pytest
        def test_some(tmpdir):
            assert tmpdir.isdir()
    """)
    reprec = testdir.inline_run()
    reprec.assertoutcome(passed=1)


@pytest.mark.usefixtures("break_getuser")
@pytest.mark.skipif(sys.platform.startswith('win'), reason='no os.getuid on windows')
def test_get_user_uid_not_found():
    """Test that get_user() function works even if the current process's
    user id does not correspond to a valid user (e.g. running pytest in a
    Docker container with 'docker run -u'.
    """
    from _pytest.tmpdir import get_user
    assert get_user() is None


@pytest.mark.skipif(not sys.platform.startswith('win'), reason='win only')
def test_get_user(monkeypatch):
    """Test that get_user() function works even if environment variables
    required by getpass module are missing from the environment on Windows
    (#1010).
    """
    from _pytest.tmpdir import get_user
    monkeypatch.delenv('USER', raising=False)
    monkeypatch.delenv('USERNAME', raising=False)
    assert get_user() is None
