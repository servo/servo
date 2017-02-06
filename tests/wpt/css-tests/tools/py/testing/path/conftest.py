import py
import sys
from py._path import svnwc as svncommon

svnbin = py.path.local.sysfind('svn')
repodump = py.path.local(__file__).dirpath('repotest.dump')
from py.builtin import print_

def pytest_funcarg__repowc1(request):
    if svnbin is None:
        py.test.skip("svn binary not found")

    tmpdir = request.getfuncargvalue("tmpdir")
    repo, repourl, wc = request.cached_setup(
        setup=lambda: getrepowc(tmpdir, "path1repo", "path1wc"),
        scope="module",
    )
    for x in ('test_remove', 'test_move', 'test_status_deleted'):
        if request.function.__name__.startswith(x):
            #print >>sys.stderr, ("saving repo", repo, "for", request.function)
            _savedrepowc = save_repowc(repo, wc)
            request.addfinalizer(lambda: restore_repowc(_savedrepowc))
    return repo, repourl, wc

def pytest_funcarg__repowc2(request):
    tmpdir = request.getfuncargvalue("tmpdir")
    name = request.function.__name__
    repo, url, wc = getrepowc(tmpdir, "%s-repo-2" % name, "%s-wc-2" % name)
    return repo, url, wc

def getsvnbin():
    if svnbin is None:
        py.test.skip("svn binary not found")
    return svnbin

# make a wc directory out of a given root url
# cache previously obtained wcs!
#
def getrepowc(tmpdir, reponame='basetestrepo', wcname='wc'):
    repo = tmpdir.mkdir(reponame)
    wcdir = tmpdir.mkdir(wcname)
    repo.ensure(dir=1)
    py.process.cmdexec('svnadmin create "%s"' %
            svncommon._escape_helper(repo))
    py.process.cmdexec('svnadmin load -q "%s" <"%s"' %
            (svncommon._escape_helper(repo), repodump))
    print_("created svn repository", repo)
    wcdir.ensure(dir=1)
    wc = py.path.svnwc(wcdir)
    if py.std.sys.platform == 'win32':
        repourl = "file://" + '/' + str(repo).replace('\\', '/')
    else:
        repourl = "file://%s" % repo
    wc.checkout(repourl)
    print_("checked out new repo into", wc)
    return (repo, repourl, wc)


def save_repowc(repo, wc):
    assert not str(repo).startswith("file://"), repo
    assert repo.check()
    savedrepo = repo.dirpath(repo.basename+".1")
    savedwc = wc.dirpath(wc.basename+".1")
    repo.copy(savedrepo)
    wc.localpath.copy(savedwc.localpath)
    return savedrepo, savedwc

def restore_repowc(obj):
    savedrepo, savedwc = obj
    #print >>sys.stderr, ("restoring", savedrepo)
    repo = savedrepo.new(basename=savedrepo.basename[:-2])
    assert repo.check()
    wc = savedwc.new(basename=savedwc.basename[:-2])
    assert wc.check()
    wc.localpath.remove()
    repo.remove()
    savedrepo.move(repo)
    savedwc.localpath.move(wc.localpath)
    py.path.svnurl._lsnorevcache.clear()
    py.path.svnurl._lsrevcache.clear()
