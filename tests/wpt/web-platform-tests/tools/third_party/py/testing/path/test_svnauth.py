import py
from py.path import SvnAuth
import time
import sys

svnbin = py.path.local.sysfind('svn')


def make_repo_auth(repo, userdata):
    """ write config to repo

        user information in userdata is used for auth
        userdata has user names as keys, and a tuple (password, readwrite) as
        values, where 'readwrite' is either 'r' or 'rw'
    """
    confdir = py.path.local(repo).join('conf')
    confdir.join('svnserve.conf').write('''\
[general]
anon-access = none
password-db = passwd
authz-db = authz
realm = TestRepo
''')
    authzdata = '[/]\n'
    passwddata = '[users]\n'
    for user in userdata:
        authzdata += '%s = %s\n' % (user, userdata[user][1])
        passwddata += '%s = %s\n' % (user, userdata[user][0])
    confdir.join('authz').write(authzdata)
    confdir.join('passwd').write(passwddata)

def serve_bg(repopath):
    pidfile = py.path.local(repopath).join('pid')
    port = 10000
    e = None
    while port < 10010:
        cmd = 'svnserve -d -T --listen-port=%d --pid-file=%s -r %s' % (
               port, pidfile, repopath)
        print(cmd)
        try:
            py.process.cmdexec(cmd)
        except py.process.cmdexec.Error:
            e = sys.exc_info()[1]
        else:
            # XXX we assume here that the pid file gets written somewhere, I
            # guess this should be relatively safe... (I hope, at least?)
            counter = pid = 0
            while counter < 10:
                counter += 1
                try:
                    pid = pidfile.read()
                except py.error.ENOENT:
                    pass
                if pid:
                    break
                time.sleep(0.2)
            return port, int(pid)
        port += 1
    raise IOError('could not start svnserve: %s' % (e,))

class TestSvnAuth(object):
    def test_basic(self):
        auth = SvnAuth('foo', 'bar')
        assert auth.username == 'foo'
        assert auth.password == 'bar'
        assert str(auth)

    def test_makecmdoptions_uname_pw_makestr(self):
        auth = SvnAuth('foo', 'bar')
        assert auth.makecmdoptions() == '--username="foo" --password="bar"'

    def test_makecmdoptions_quote_escape(self):
        auth = SvnAuth('fo"o', '"ba\'r"')
        assert auth.makecmdoptions() == '--username="fo\\"o" --password="\\"ba\'r\\""'

    def test_makecmdoptions_no_cache_auth(self):
        auth = SvnAuth('foo', 'bar', cache_auth=False)
        assert auth.makecmdoptions() == ('--username="foo" --password="bar" '
                                         '--no-auth-cache')

    def test_makecmdoptions_no_interactive(self):
        auth = SvnAuth('foo', 'bar', interactive=False)
        assert auth.makecmdoptions() == ('--username="foo" --password="bar" '
                                         '--non-interactive')

    def test_makecmdoptions_no_interactive_no_cache_auth(self):
        auth = SvnAuth('foo', 'bar', cache_auth=False,
                               interactive=False)
        assert auth.makecmdoptions() == ('--username="foo" --password="bar" '
                                         '--no-auth-cache --non-interactive')

class svnwc_no_svn(py.path.svnwc):
    def __new__(cls, *args, **kwargs):
        self = super(svnwc_no_svn, cls).__new__(cls, *args, **kwargs)
        self.commands = []
        return self

    def _svn(self, *args):
        self.commands.append(args)

class TestSvnWCAuth(object):
    def setup_method(self, meth):
        if not svnbin:
            py.test.skip("svn binary required")
        self.auth = SvnAuth('user', 'pass', cache_auth=False)

    def test_checkout(self):
        wc = svnwc_no_svn('foo', auth=self.auth)
        wc.checkout('url')
        assert wc.commands[0][-1] == ('--username="user" --password="pass" '
                                      '--no-auth-cache')

    def test_commit(self):
        wc = svnwc_no_svn('foo', auth=self.auth)
        wc.commit('msg')
        assert wc.commands[0][-1] == ('--username="user" --password="pass" '
                                      '--no-auth-cache')

    def test_checkout_no_cache_auth(self):
        wc = svnwc_no_svn('foo', auth=self.auth)
        wc.checkout('url')
        assert wc.commands[0][-1] == ('--username="user" --password="pass" '
                                      '--no-auth-cache')

    def test_checkout_auth_from_constructor(self):
        wc = svnwc_no_svn('foo', auth=self.auth)
        wc.checkout('url')
        assert wc.commands[0][-1] == ('--username="user" --password="pass" '
                                      '--no-auth-cache')

class svnurl_no_svn(py.path.svnurl):
    cmdexec_output = 'test'
    popen_output = 'test'
    def __new__(cls, *args, **kwargs):
        self = super(svnurl_no_svn, cls).__new__(cls, *args, **kwargs)
        self.commands = []
        return self

    def _cmdexec(self, cmd):
        self.commands.append(cmd)
        return self.cmdexec_output

    def _popen(self, cmd):
        self.commands.append(cmd)
        return self.popen_output

class TestSvnURLAuth(object):
    def setup_method(self, meth):
        self.auth = SvnAuth('foo', 'bar')

    def test_init(self):
        u = svnurl_no_svn('http://foo.bar/svn')
        assert u.auth is None

        u = svnurl_no_svn('http://foo.bar/svn', auth=self.auth)
        assert u.auth is self.auth

    def test_new(self):
        u = svnurl_no_svn('http://foo.bar/svn/foo', auth=self.auth)
        new = u.new(basename='bar')
        assert new.auth is self.auth
        assert new.url == 'http://foo.bar/svn/bar'

    def test_join(self):
        u = svnurl_no_svn('http://foo.bar/svn', auth=self.auth)
        new = u.join('foo')
        assert new.auth is self.auth
        assert new.url == 'http://foo.bar/svn/foo'

    def test_listdir(self):
        u = svnurl_no_svn('http://foo.bar/svn', auth=self.auth)
        u.cmdexec_output = '''\
   1717 johnny           1529 Nov 04 14:32 LICENSE.txt
   1716 johnny           5352 Nov 04 14:28 README.txt
'''
        paths = u.listdir()
        assert paths[0].auth is self.auth
        assert paths[1].auth is self.auth
        assert paths[0].basename == 'LICENSE.txt'

    def test_info(self):
        u = svnurl_no_svn('http://foo.bar/svn/LICENSE.txt', auth=self.auth)
        def dirpath(self):
            return self
        u.cmdexec_output = '''\
   1717 johnny           1529 Nov 04 14:32 LICENSE.txt
   1716 johnny           5352 Nov 04 14:28 README.txt
'''
        org_dp = u.__class__.dirpath
        u.__class__.dirpath = dirpath
        try:
            info = u.info()
        finally:
            u.dirpath = org_dp
        assert info.size == 1529

    def test_open(self):
        u = svnurl_no_svn('http://foo.bar/svn', auth=self.auth)
        foo = u.join('foo')
        foo.check = lambda *args, **kwargs: True
        ret = foo.open()
        assert ret == 'test'
        assert '--username="foo" --password="bar"' in foo.commands[0]

    def test_dirpath(self):
        u = svnurl_no_svn('http://foo.bar/svn/foo', auth=self.auth)
        parent = u.dirpath()
        assert parent.auth is self.auth

    def test_mkdir(self):
        u = svnurl_no_svn('http://foo.bar/svn/qweqwe', auth=self.auth)
        assert not u.commands
        u.mkdir(msg='created dir foo')
        assert u.commands
        assert '--username="foo" --password="bar"' in u.commands[0]

    def test_copy(self):
        u = svnurl_no_svn('http://foo.bar/svn', auth=self.auth)
        u2 = svnurl_no_svn('http://foo.bar/svn2')
        u.copy(u2, 'copied dir')
        assert '--username="foo" --password="bar"' in u.commands[0]

    def test_rename(self):
        u = svnurl_no_svn('http://foo.bar/svn/foo', auth=self.auth)
        u.rename('http://foo.bar/svn/bar', 'moved foo to bar')
        assert '--username="foo" --password="bar"' in u.commands[0]

    def test_remove(self):
        u = svnurl_no_svn('http://foo.bar/svn/foo', auth=self.auth)
        u.remove(msg='removing foo')
        assert '--username="foo" --password="bar"' in u.commands[0]

    def test_export(self):
        u = svnurl_no_svn('http://foo.bar/svn', auth=self.auth)
        target = py.path.local('/foo')
        u.export(target)
        assert '--username="foo" --password="bar"' in u.commands[0]

    def test_log(self):
        u = svnurl_no_svn('http://foo.bar/svn/foo', auth=self.auth)
        u.popen_output = py.io.TextIO(py.builtin._totext('''\
<?xml version="1.0"?>
<log>
<logentry revision="51381">
<author>guido</author>
<date>2008-02-11T12:12:18.476481Z</date>
<msg>Creating branch to work on auth support for py.path.svn*.
</msg>
</logentry>
</log>
''', 'ascii'))
        u.check = lambda *args, **kwargs: True
        ret = u.log(10, 20, verbose=True)
        assert '--username="foo" --password="bar"' in u.commands[0]
        assert len(ret) == 1
        assert int(ret[0].rev) == 51381
        assert ret[0].author == 'guido'

    def test_propget(self):
        u = svnurl_no_svn('http://foo.bar/svn', auth=self.auth)
        u.propget('foo')
        assert '--username="foo" --password="bar"' in u.commands[0]

def pytest_funcarg__setup(request):
    return Setup(request)

class Setup:
    def __init__(self, request):
        if not svnbin:
            py.test.skip("svn binary required")
        if not request.config.option.runslowtests:
            py.test.skip('use --runslowtests to run these tests')

        tmpdir = request.getfuncargvalue("tmpdir")
        repodir = tmpdir.join("repo")
        py.process.cmdexec('svnadmin create %s' % repodir)
        if sys.platform == 'win32':
            repodir = '/' + str(repodir).replace('\\', '/')
        self.repo = py.path.svnurl("file://%s" % repodir)
        if sys.platform == 'win32':
            # remove trailing slash...
            repodir = repodir[1:]
        self.repopath = py.path.local(repodir)
        self.temppath = tmpdir.mkdir("temppath")
        self.auth = SvnAuth('johnny', 'foo', cache_auth=False,
                                    interactive=False)
        make_repo_auth(self.repopath, {'johnny': ('foo', 'rw')})
        self.port, self.pid = serve_bg(self.repopath.dirpath())
        # XXX caching is too global
        py.path.svnurl._lsnorevcache._dict.clear()
        request.addfinalizer(lambda: py.process.kill(self.pid))

class TestSvnWCAuthFunctional:
    def test_checkout_constructor_arg(self, setup):
        wc = py.path.svnwc(setup.temppath, auth=setup.auth)
        wc.checkout(
            'svn://localhost:%s/%s' % (setup.port, setup.repopath.basename))
        assert wc.join('.svn').check()

    def test_checkout_function_arg(self, setup):
        wc = py.path.svnwc(setup.temppath, auth=setup.auth)
        wc.checkout(
            'svn://localhost:%s/%s' % (setup.port, setup.repopath.basename))
        assert wc.join('.svn').check()

    def test_checkout_failing_non_interactive(self, setup):
        auth = SvnAuth('johnny', 'bar', cache_auth=False,
                               interactive=False)
        wc = py.path.svnwc(setup.temppath, auth)
        py.test.raises(Exception,
           ("wc.checkout('svn://localhost:%(port)s/%(repopath)s')" %
             setup.__dict__))

    def test_log(self, setup):
        wc = py.path.svnwc(setup.temppath, setup.auth)
        wc.checkout(
            'svn://localhost:%s/%s' % (setup.port, setup.repopath.basename))
        foo = wc.ensure('foo.txt')
        wc.commit('added foo.txt')
        log = foo.log()
        assert len(log) == 1
        assert log[0].msg == 'added foo.txt'

    def test_switch(self, setup):
        import pytest
        try:
            import xdist
            pytest.skip('#160: fails under xdist')
        except ImportError:
            pass
        wc = py.path.svnwc(setup.temppath, auth=setup.auth)
        svnurl = 'svn://localhost:%s/%s' % (setup.port, setup.repopath.basename)
        wc.checkout(svnurl)
        wc.ensure('foo', dir=True).ensure('foo.txt').write('foo')
        wc.commit('added foo dir with foo.txt file')
        wc.ensure('bar', dir=True)
        wc.commit('added bar dir')
        bar = wc.join('bar')
        bar.switch(svnurl + '/foo')
        assert bar.join('foo.txt')

    def test_update(self, setup):
        wc1 = py.path.svnwc(setup.temppath.ensure('wc1', dir=True),
                            auth=setup.auth)
        wc2 = py.path.svnwc(setup.temppath.ensure('wc2', dir=True),
                            auth=setup.auth)
        wc1.checkout(
            'svn://localhost:%s/%s' % (setup.port, setup.repopath.basename))
        wc2.checkout(
            'svn://localhost:%s/%s' % (setup.port, setup.repopath.basename))
        wc1.ensure('foo', dir=True)
        wc1.commit('added foo dir')
        wc2.update()
        assert wc2.join('foo').check()

        auth = SvnAuth('unknown', 'unknown', interactive=False)
        wc2.auth = auth
        py.test.raises(Exception, 'wc2.update()')

    def test_lock_unlock_status(self, setup):
        port = setup.port
        wc = py.path.svnwc(setup.temppath, auth=setup.auth)
        wc.checkout(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename,))
        wc.ensure('foo', file=True)
        wc.commit('added foo file')
        foo = wc.join('foo')
        foo.lock()
        status = foo.status()
        assert status.locked
        foo.unlock()
        status = foo.status()
        assert not status.locked

        auth = SvnAuth('unknown', 'unknown', interactive=False)
        foo.auth = auth
        py.test.raises(Exception, 'foo.lock()')
        py.test.raises(Exception, 'foo.unlock()')

    def test_diff(self, setup):
        port = setup.port
        wc = py.path.svnwc(setup.temppath, auth=setup.auth)
        wc.checkout(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename,))
        wc.ensure('foo', file=True)
        wc.commit('added foo file')
        wc.update()
        rev = int(wc.status().rev)
        foo = wc.join('foo')
        foo.write('bar')
        diff = foo.diff()
        assert '\n+bar\n' in diff
        foo.commit('added some content')
        diff = foo.diff()
        assert not diff
        diff = foo.diff(rev=rev)
        assert '\n+bar\n' in diff

        auth = SvnAuth('unknown', 'unknown', interactive=False)
        foo.auth = auth
        py.test.raises(Exception, 'foo.diff(rev=rev)')

class TestSvnURLAuthFunctional:
    def test_listdir(self, setup):
        port = setup.port
        u = py.path.svnurl(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename),
            auth=setup.auth)
        u.ensure('foo')
        paths = u.listdir()
        assert len(paths) == 1
        assert paths[0].auth is setup.auth

        auth = SvnAuth('foo', 'bar', interactive=False)
        u = py.path.svnurl(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename),
            auth=auth)
        py.test.raises(Exception, 'u.listdir()')

    def test_copy(self, setup):
        port = setup.port
        u = py.path.svnurl(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename),
            auth=setup.auth)
        foo = u.mkdir('foo')
        assert foo.check()
        bar = u.join('bar')
        foo.copy(bar)
        assert bar.check()
        assert bar.auth is setup.auth

        auth = SvnAuth('foo', 'bar', interactive=False)
        u = py.path.svnurl(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename),
            auth=auth)
        foo = u.join('foo')
        bar = u.join('bar')
        py.test.raises(Exception, 'foo.copy(bar)')

    def test_write_read(self, setup):
        port = setup.port
        u = py.path.svnurl(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename),
            auth=setup.auth)
        foo = u.ensure('foo')
        fp = foo.open()
        try:
            data = fp.read()
        finally:
            fp.close()
        assert data == ''

        auth = SvnAuth('foo', 'bar', interactive=False)
        u = py.path.svnurl(
            'svn://localhost:%s/%s' % (port, setup.repopath.basename),
            auth=auth)
        foo = u.join('foo')
        py.test.raises(Exception, 'foo.open()')

    # XXX rinse, repeat... :|
