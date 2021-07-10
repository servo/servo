import py
import os, sys
import pytest
from py._path.svnwc import InfoSvnWCCommand, XMLWCStatus, parse_wcinfotime
from py._path import svnwc as svncommon
from svntestbase import CommonSvnTests


pytestmark = pytest.mark.xfail(sys.platform.startswith('win'),
                               reason='#161 all tests in this file are failing on Windows',
                               run=False)


def test_make_repo(path1, tmpdir):
    repo = tmpdir.join("repo")
    py.process.cmdexec('svnadmin create %s' % repo)
    if sys.platform == 'win32':
        repo = '/' + str(repo).replace('\\', '/')
    repo = py.path.svnurl("file://%s" % repo)
    wc = py.path.svnwc(tmpdir.join("wc"))
    wc.checkout(repo)
    assert wc.rev == 0
    assert len(wc.listdir()) == 0
    p = wc.join("a_file")
    p.write("test file")
    p.add()
    rev = wc.commit("some test")
    assert p.info().rev == 1
    assert rev == 1
    rev = wc.commit()
    assert rev is None

def pytest_funcarg__path1(request):
    repo, repourl, wc = request.getfuncargvalue("repowc1")
    return wc

class TestWCSvnCommandPath(CommonSvnTests):
    def test_status_attributes_simple(self, path1):
        def assert_nochange(p):
            s = p.status()
            assert not s.modified
            assert not s.prop_modified
            assert not s.added
            assert not s.deleted
            assert not s.replaced

        dpath = path1.join('sampledir')
        assert_nochange(path1.join('sampledir'))
        assert_nochange(path1.join('samplefile'))

    def test_status_added(self, path1):
        nf = path1.join('newfile')
        nf.write('hello')
        nf.add()
        try:
            s = nf.status()
            assert s.added
            assert not s.modified
            assert not s.prop_modified
            assert not s.replaced
        finally:
            nf.revert()

    def test_status_change(self, path1):
        nf = path1.join('samplefile')
        try:
            nf.write(nf.read() + 'change')
            s = nf.status()
            assert not s.added
            assert s.modified
            assert not s.prop_modified
            assert not s.replaced
        finally:
            nf.revert()

    def test_status_added_ondirectory(self, path1):
        sampledir = path1.join('sampledir')
        try:
            t2 = sampledir.mkdir('t2')
            t1 = t2.join('t1')
            t1.write('test')
            t1.add()
            s = sampledir.status(rec=1)
            # Comparing just the file names, because paths are unpredictable
            # on Windows. (long vs. 8.3 paths)
            assert t1.basename in [item.basename for item in s.added]
            assert t2.basename in [item.basename for item in s.added]
        finally:
            t2.revert(rec=1)
            t2.localpath.remove(rec=1)

    def test_status_unknown(self, path1):
        t1 = path1.join('un1')
        try:
            t1.write('test')
            s = path1.status()
            # Comparing just the file names, because paths are unpredictable
            # on Windows. (long vs. 8.3 paths)
            assert t1.basename in [item.basename for item in s.unknown]
        finally:
            t1.localpath.remove()

    def test_status_unchanged(self, path1):
        r = path1
        s = path1.status(rec=1)
        # Comparing just the file names, because paths are unpredictable
        # on Windows. (long vs. 8.3 paths)
        assert r.join('samplefile').basename in [item.basename
                                                    for item in s.unchanged]
        assert r.join('sampledir').basename in [item.basename
                                                    for item in s.unchanged]
        assert r.join('sampledir/otherfile').basename in [item.basename
                                                    for item in s.unchanged]

    def test_status_update(self, path1):
        # not a mark because the global "pytestmark" will end up overwriting a mark here
        pytest.xfail("svn-1.7 has buggy 'status --xml' output")
        r = path1
        try:
            r.update(rev=1)
            s = r.status(updates=1, rec=1)
            # Comparing just the file names, because paths are unpredictable
            # on Windows. (long vs. 8.3 paths)
            import pprint
            pprint.pprint(s.allpath())
            assert r.join('anotherfile').basename in [item.basename for
                                                    item in s.update_available]
            #assert len(s.update_available) == 1
        finally:
            r.update()

    def test_status_replaced(self, path1):
        p = path1.join("samplefile")
        p.remove()
        p.ensure(dir=0)
        try:
            s = path1.status()
            assert p.basename in [item.basename for item in s.replaced]
        finally:
            path1.revert(rec=1)

    def test_status_ignored(self, path1):
        try:
            d = path1.join('sampledir')
            p = py.path.local(d).join('ignoredfile')
            p.ensure(file=True)
            s = d.status()
            assert [x.basename for x in s.unknown] == ['ignoredfile']
            assert [x.basename for x in s.ignored] == []
            d.propset('svn:ignore', 'ignoredfile')
            s = d.status()
            assert [x.basename for x in s.unknown] == []
            assert [x.basename for x in s.ignored] == ['ignoredfile']
        finally:
            path1.revert(rec=1)

    def test_status_conflict(self, path1, tmpdir):
        wc = path1
        wccopy = py.path.svnwc(tmpdir.join("conflict_copy"))
        wccopy.checkout(wc.url)
        p = wc.ensure('conflictsamplefile', file=1)
        p.write('foo')
        wc.commit('added conflictsamplefile')
        wccopy.update()
        assert wccopy.join('conflictsamplefile').check()
        p.write('bar')
        wc.commit('wrote some data')
        wccopy.join('conflictsamplefile').write('baz')
        wccopy.update(interactive=False)
        s = wccopy.status()
        assert [x.basename for x in s.conflict] == ['conflictsamplefile']

    def test_status_external(self, path1, repowc2):
        otherrepo, otherrepourl, otherwc = repowc2
        d = path1.ensure('sampledir', dir=1)
        try:
            d.update()
            d.propset('svn:externals', 'otherwc %s' % (otherwc.url,))
            d.update()
            s = d.status()
            assert [x.basename for x in s.external] == ['otherwc']
            assert 'otherwc' not in [x.basename for x in s.unchanged]
            s = d.status(rec=1)
            assert [x.basename for x in s.external] == ['otherwc']
            assert 'otherwc' in [x.basename for x in s.unchanged]
        finally:
            path1.revert(rec=1)

    def test_status_deleted(self, path1):
        d = path1.ensure('sampledir', dir=1)
        d.remove()
        d.ensure(dir=1)
        path1.commit()
        d.ensure('deletefile', dir=0)
        d.commit()
        s = d.status()
        assert 'deletefile' in [x.basename for x in s.unchanged]
        assert not s.deleted
        p = d.join('deletefile')
        p.remove()
        s = d.status()
        assert 'deletefile' not in s.unchanged
        assert [x.basename for x in s.deleted] == ['deletefile']

    def test_status_noauthor(self, path1):
        # testing for XML without author - this used to raise an exception
        xml = '''\
        <entry path="/tmp/pytest-23/wc">
        <wc-status item="normal" props="none" revision="0">
        <commit revision="0">
        <date>2008-08-19T16:50:53.400198Z</date>
        </commit>
        </wc-status>
        </entry>
        '''
        XMLWCStatus.fromstring(xml, path1)

    def test_status_wrong_xml(self, path1):
        # testing for XML without author - this used to raise an exception
        xml = '<entry path="/home/jean/zope/venv/projectdb/parts/development-products/DataGridField">\n<wc-status item="incomplete" props="none" revision="784">\n</wc-status>\n</entry>'
        st = XMLWCStatus.fromstring(xml, path1)
        assert len(st.incomplete) == 1

    def test_diff(self, path1):
        p = path1 / 'anotherfile'
        out = p.diff(rev=2)
        assert out.find('hello') != -1

    def test_blame(self, path1):
        p = path1.join('samplepickle')
        lines = p.blame()
        assert sum([l[0] for l in lines]) == len(lines)
        for l1, l2 in zip(p.readlines(), [l[2] for l in lines]):
            assert l1 == l2
        assert [l[1] for l in lines] == ['hpk'] * len(lines)
        p = path1.join('samplefile')
        lines = p.blame()
        assert sum([l[0] for l in lines]) == len(lines)
        for l1, l2 in zip(p.readlines(), [l[2] for l in lines]):
            assert l1 == l2
        assert [l[1] for l in lines] == ['hpk'] * len(lines)

    def test_join_abs(self, path1):
        s = str(path1.localpath)
        n = path1.join(s, abs=1)
        assert path1 == n

    def test_join_abs2(self, path1):
        assert path1.join('samplefile', abs=1) == path1.join('samplefile')

    def test_str_gives_localpath(self, path1):
        assert str(path1) == str(path1.localpath)

    def test_versioned(self, path1):
        assert path1.check(versioned=1)
        # TODO: Why does my copy of svn think .svn is versioned?
        #assert path1.join('.svn').check(versioned=0)
        assert path1.join('samplefile').check(versioned=1)
        assert not path1.join('notexisting').check(versioned=1)
        notexisting = path1.join('hello').localpath
        try:
            notexisting.write("")
            assert path1.join('hello').check(versioned=0)
        finally:
            notexisting.remove()

    def test_listdir_versioned(self, path1):
        assert path1.check(versioned=1)
        p = path1.localpath.ensure("not_a_versioned_file")
        l = [x.localpath
                for x in path1.listdir(lambda x: x.check(versioned=True))]
        assert p not in l

    def test_nonversioned_remove(self, path1):
        assert path1.check(versioned=1)
        somefile = path1.join('nonversioned/somefile')
        nonwc = py.path.local(somefile)
        nonwc.ensure()
        assert somefile.check()
        assert not somefile.check(versioned=True)
        somefile.remove() # this used to fail because it tried to 'svn rm'

    def test_properties(self, path1):
        try:
            path1.propset('gaga', 'this')
            assert path1.propget('gaga') == 'this'
            # Comparing just the file names, because paths are unpredictable
            # on Windows. (long vs. 8.3 paths)
            assert path1.basename in [item.basename for item in
                                        path1.status().prop_modified]
            assert 'gaga' in path1.proplist()
            assert path1.proplist()['gaga'] == 'this'

        finally:
            path1.propdel('gaga')

    def test_proplist_recursive(self, path1):
        s = path1.join('samplefile')
        s.propset('gugu', 'that')
        try:
            p = path1.proplist(rec=1)
            # Comparing just the file names, because paths are unpredictable
            # on Windows. (long vs. 8.3 paths)
            assert (path1 / 'samplefile').basename in [item.basename
                                                                for item in p]
        finally:
            s.propdel('gugu')

    def test_long_properties(self, path1):
        value = """
        vadm:posix : root root 0100755
        Properties on 'chroot/dns/var/bind/db.net.xots':
                """
        try:
            path1.propset('gaga', value)
            backvalue = path1.propget('gaga')
            assert backvalue == value
            #assert len(backvalue.split('\n')) == 1
        finally:
            path1.propdel('gaga')


    def test_ensure(self, path1):
        newpath = path1.ensure('a', 'b', 'c')
        try:
            assert newpath.check(exists=1, versioned=1)
            newpath.write("hello")
            newpath.ensure()
            assert newpath.read() == "hello"
        finally:
            path1.join('a').remove(force=1)

    def test_not_versioned(self, path1):
        p = path1.localpath.mkdir('whatever')
        f = path1.localpath.ensure('testcreatedfile')
        try:
            assert path1.join('whatever').check(versioned=0)
            assert path1.join('testcreatedfile').check(versioned=0)
            assert not path1.join('testcreatedfile').check(versioned=1)
        finally:
            p.remove(rec=1)
            f.remove()

    def test_lock_unlock(self, path1):
        root = path1
        somefile = root.join('somefile')
        somefile.ensure(file=True)
        # not yet added to repo
        py.test.raises(Exception, 'somefile.lock()')
        somefile.write('foo')
        somefile.commit('test')
        assert somefile.check(versioned=True)
        somefile.lock()
        try:
            locked = root.status().locked
            assert len(locked) == 1
            assert locked[0].basename == somefile.basename
            assert locked[0].dirpath().basename == somefile.dirpath().basename
            #assert somefile.locked()
            py.test.raises(Exception, 'somefile.lock()')
        finally:
            somefile.unlock()
        #assert not somefile.locked()
        locked = root.status().locked
        assert locked == []
        py.test.raises(Exception, 'somefile,unlock()')
        somefile.remove()

    def test_commit_nonrecursive(self, path1):
        somedir = path1.join('sampledir')
        somedir.mkdir("subsubdir")
        somedir.propset('foo', 'bar')
        status = somedir.status()
        assert len(status.prop_modified) == 1
        assert len(status.added) == 1

        somedir.commit('non-recursive commit', rec=0)
        status = somedir.status()
        assert len(status.prop_modified) == 0
        assert len(status.added) == 1

        somedir.commit('recursive commit')
        status = somedir.status()
        assert len(status.prop_modified) == 0
        assert len(status.added) == 0

    def test_commit_return_value(self, path1):
        testfile = path1.join('test.txt').ensure(file=True)
        testfile.write('test')
        rev = path1.commit('testing')
        assert type(rev) == int

        anotherfile = path1.join('another.txt').ensure(file=True)
        anotherfile.write('test')
        rev2 = path1.commit('testing more')
        assert type(rev2) == int
        assert rev2 == rev + 1

    #def test_log(self, path1):
    #   l = path1.log()
    #   assert len(l) == 3  # might need to be upped if more tests are added

class XTestWCSvnCommandPathSpecial:

    rooturl = 'http://codespeak.net/svn/py.path/trunk/dist/py.path/test/data'
    #def test_update_none_rev(self, path1):
    #    path = tmpdir.join('checkouttest')
    #    wcpath = newpath(xsvnwc=str(path), url=path1url)
    #    try:
    #        wcpath.checkout(rev=2100)
    #        wcpath.update()
    #        assert wcpath.info().rev > 2100
    #    finally:
    #        wcpath.localpath.remove(rec=1)

def test_parse_wcinfotime():
    assert (parse_wcinfotime('2006-05-30 20:45:26 +0200 (Tue, 30 May 2006)') ==
            1149021926)
    assert (parse_wcinfotime('2003-10-27 20:43:14 +0100 (Mon, 27 Oct 2003)') ==
            1067287394)

class TestInfoSvnWCCommand:

    def test_svn_1_2(self, path1):
        output = """
        Path: test_svnwc.py
        Name: test_svnwc.py
        URL: http://codespeak.net/svn/py/dist/py/path/svn/wccommand.py
        Repository UUID: fd0d7bf2-dfb6-0310-8d31-b7ecfe96aada
        Revision: 28137
        Node Kind: file
        Schedule: normal
        Last Changed Author: jan
        Last Changed Rev: 27939
        Last Changed Date: 2006-05-30 20:45:26 +0200 (Tue, 30 May 2006)
        Text Last Updated: 2006-06-01 00:42:53 +0200 (Thu, 01 Jun 2006)
        Properties Last Updated: 2006-05-23 11:54:59 +0200 (Tue, 23 May 2006)
        Checksum: 357e44880e5d80157cc5fbc3ce9822e3
        """
        path = py.path.local(__file__).dirpath().chdir()
        try:
            info = InfoSvnWCCommand(output)
        finally:
            path.chdir()
        assert info.last_author == 'jan'
        assert info.kind == 'file'
        assert info.mtime == 1149021926.0
        assert info.url == 'http://codespeak.net/svn/py/dist/py/path/svn/wccommand.py'
        assert info.time == 1149021926000000.0
        assert info.rev == 28137


    def test_svn_1_3(self, path1):
        output = """
        Path: test_svnwc.py
        Name: test_svnwc.py
        URL: http://codespeak.net/svn/py/dist/py/path/svn/wccommand.py
        Repository Root: http://codespeak.net/svn
        Repository UUID: fd0d7bf2-dfb6-0310-8d31-b7ecfe96aada
        Revision: 28124
        Node Kind: file
        Schedule: normal
        Last Changed Author: jan
        Last Changed Rev: 27939
        Last Changed Date: 2006-05-30 20:45:26 +0200 (Tue, 30 May 2006)
        Text Last Updated: 2006-06-02 23:46:11 +0200 (Fri, 02 Jun 2006)
        Properties Last Updated: 2006-06-02 23:45:28 +0200 (Fri, 02 Jun 2006)
        Checksum: 357e44880e5d80157cc5fbc3ce9822e3
        """
        path = py.path.local(__file__).dirpath().chdir()
        try:
            info = InfoSvnWCCommand(output)
        finally:
            path.chdir()
        assert info.last_author == 'jan'
        assert info.kind == 'file'
        assert info.mtime == 1149021926.0
        assert info.url == 'http://codespeak.net/svn/py/dist/py/path/svn/wccommand.py'
        assert info.rev == 28124
        assert info.time == 1149021926000000.0


def test_characters_at():
    py.test.raises(ValueError, "py.path.svnwc('/tmp/@@@:')")

def test_characters_tilde():
    py.path.svnwc('/tmp/test~')


class TestRepo:
    def test_trailing_slash_is_stripped(self, path1):
        # XXX we need to test more normalizing properties
        url = path1.join("/")
        assert path1 == url

    #def test_different_revs_compare_unequal(self, path1):
    #    newpath = path1.new(rev=1199)
    #    assert newpath != path1

    def test_exists_svn_root(self, path1):
        assert path1.check()

    #def test_not_exists_rev(self, path1):
    #    url = path1.__class__(path1url, rev=500)
    #    assert url.check(exists=0)

    #def test_nonexisting_listdir_rev(self, path1):
    #    url = path1.__class__(path1url, rev=500)
    #    raises(py.error.ENOENT, url.listdir)

    #def test_newrev(self, path1):
    #    url = path1.new(rev=None)
    #    assert url.rev == None
    #    assert url.strpath == path1.strpath
    #    url = path1.new(rev=10)
    #    assert url.rev == 10

    #def test_info_rev(self, path1):
    #    url = path1.__class__(path1url, rev=1155)
    #    url = url.join("samplefile")
    #    res = url.info()
    #    assert res.size > len("samplefile") and res.created_rev == 1155

    # the following tests are easier if we have a path class
    def test_repocache_simple(self, path1):
        repocache = svncommon.RepoCache()
        repocache.put(path1.strpath, 42)
        url, rev = repocache.get(path1.join('test').strpath)
        assert rev == 42
        assert url == path1.strpath

    def test_repocache_notimeout(self, path1):
        repocache = svncommon.RepoCache()
        repocache.timeout = 0
        repocache.put(path1.strpath, path1.rev)
        url, rev = repocache.get(path1.strpath)
        assert rev == -1
        assert url == path1.strpath

    def test_repocache_outdated(self, path1):
        repocache = svncommon.RepoCache()
        repocache.put(path1.strpath, 42, timestamp=0)
        url, rev = repocache.get(path1.join('test').strpath)
        assert rev == -1
        assert url == path1.strpath

    def _test_getreporev(self):
        """ this test runs so slow it's usually disabled """
        old = svncommon.repositories.repos
        try:
            _repocache.clear()
            root = path1.new(rev=-1)
            url, rev = cache.repocache.get(root.strpath)
            assert rev>=0
            assert url == svnrepourl
        finally:
            repositories.repos = old
