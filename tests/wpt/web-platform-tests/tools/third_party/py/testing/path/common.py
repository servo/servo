import py
import sys

import pytest

class CommonFSTests(object):
    def test_constructor_equality(self, path1):
        p = path1.__class__(path1)
        assert p == path1

    def test_eq_nonstring(self, path1):
        p1 = path1.join('sampledir')
        p2 = path1.join('sampledir')
        assert p1 == p2

    def test_new_identical(self, path1):
        assert path1 == path1.new()

    def test_join(self, path1):
        p = path1.join('sampledir')
        strp = str(p)
        assert strp.endswith('sampledir')
        assert strp.startswith(str(path1))

    def test_join_normalized(self, path1):
        newpath = path1.join(path1.sep+'sampledir')
        strp = str(newpath)
        assert strp.endswith('sampledir')
        assert strp.startswith(str(path1))
        newpath = path1.join((path1.sep*2) + 'sampledir')
        strp = str(newpath)
        assert strp.endswith('sampledir')
        assert strp.startswith(str(path1))

    def test_join_noargs(self, path1):
        newpath = path1.join()
        assert path1 == newpath

    def test_add_something(self, path1):
        p = path1.join('sample')
        p = p + 'dir'
        assert p.check()
        assert p.exists()
        assert p.isdir()
        assert not p.isfile()

    def test_parts(self, path1):
        newpath = path1.join('sampledir', 'otherfile')
        par = newpath.parts()[-3:]
        assert par == [path1, path1.join('sampledir'), newpath]

        revpar = newpath.parts(reverse=True)[:3]
        assert revpar == [newpath, path1.join('sampledir'), path1]

    def test_common(self, path1):
        other = path1.join('sampledir')
        x = other.common(path1)
        assert x == path1

    #def test_parents_nonexisting_file(self, path1):
    #    newpath = path1 / 'dirnoexist' / 'nonexisting file'
    #    par = list(newpath.parents())
    #    assert par[:2] == [path1 / 'dirnoexist', path1]

    def test_basename_checks(self, path1):
        newpath = path1.join('sampledir')
        assert newpath.check(basename='sampledir')
        assert newpath.check(notbasename='xyz')
        assert newpath.basename == 'sampledir'

    def test_basename(self, path1):
        newpath = path1.join('sampledir')
        assert newpath.check(basename='sampledir')
        assert newpath.basename, 'sampledir'

    def test_dirname(self, path1):
        newpath = path1.join('sampledir')
        assert newpath.dirname == str(path1)

    def test_dirpath(self, path1):
        newpath = path1.join('sampledir')
        assert newpath.dirpath() == path1

    def test_dirpath_with_args(self, path1):
        newpath = path1.join('sampledir')
        assert newpath.dirpath('x') == path1.join('x')

    def test_newbasename(self, path1):
        newpath = path1.join('samplefile')
        newbase = newpath.new(basename="samplefile2")
        assert newbase.basename == "samplefile2"
        assert newbase.dirpath() == newpath.dirpath()

    def test_not_exists(self, path1):
        assert not path1.join('does_not_exist').check()
        assert path1.join('does_not_exist').check(exists=0)

    def test_exists(self, path1):
        assert path1.join("samplefile").check()
        assert path1.join("samplefile").check(exists=1)
        assert path1.join("samplefile").exists()
        assert path1.join("samplefile").isfile()
        assert not path1.join("samplefile").isdir()

    def test_dir(self, path1):
        #print repr(path1.join("sampledir"))
        assert path1.join("sampledir").check(dir=1)
        assert path1.join('samplefile').check(notdir=1)
        assert not path1.join("samplefile").check(dir=1)
        assert path1.join("samplefile").exists()
        assert not path1.join("samplefile").isdir()
        assert path1.join("samplefile").isfile()

    def test_fnmatch_file(self, path1):
        assert path1.join("samplefile").check(fnmatch='s*e')
        assert path1.join("samplefile").fnmatch('s*e')
        assert not path1.join("samplefile").fnmatch('s*x')
        assert not path1.join("samplefile").check(fnmatch='s*x')

    #def test_fnmatch_dir(self, path1):

    #    pattern = path1.sep.join(['s*file'])
    #    sfile = path1.join("samplefile")
    #    assert sfile.check(fnmatch=pattern)

    def test_relto(self, path1):
        l=path1.join("sampledir", "otherfile")
        assert l.relto(path1) == l.sep.join(["sampledir", "otherfile"])
        assert l.check(relto=path1)
        assert path1.check(notrelto=l)
        assert not path1.check(relto=l)

    def test_bestrelpath(self, path1):
        curdir = path1
        sep = curdir.sep
        s = curdir.bestrelpath(curdir)
        assert s == "."
        s = curdir.bestrelpath(curdir.join("hello", "world"))
        assert s == "hello" + sep + "world"

        s = curdir.bestrelpath(curdir.dirpath().join("sister"))
        assert s == ".." + sep + "sister"
        assert curdir.bestrelpath(curdir.dirpath()) == ".."

        assert curdir.bestrelpath("hello") == "hello"

    def test_relto_not_relative(self, path1):
        l1=path1.join("bcde")
        l2=path1.join("b")
        assert not l1.relto(l2)
        assert not l2.relto(l1)

    @py.test.mark.xfail("sys.platform.startswith('java')")
    def test_listdir(self, path1):
        l = path1.listdir()
        assert path1.join('sampledir') in l
        assert path1.join('samplefile') in l
        py.test.raises(py.error.ENOTDIR,
                       "path1.join('samplefile').listdir()")

    def test_listdir_fnmatchstring(self, path1):
        l = path1.listdir('s*dir')
        assert len(l)
        assert l[0], path1.join('sampledir')

    def test_listdir_filter(self, path1):
        l = path1.listdir(lambda x: x.check(dir=1))
        assert path1.join('sampledir') in l
        assert not path1.join('samplefile') in l

    def test_listdir_sorted(self, path1):
        l = path1.listdir(lambda x: x.check(basestarts="sample"), sort=True)
        assert path1.join('sampledir') == l[0]
        assert path1.join('samplefile') == l[1]
        assert path1.join('samplepickle') == l[2]

    def test_visit_nofilter(self, path1):
        l = []
        for i in path1.visit():
            l.append(i.relto(path1))
        assert "sampledir" in l
        assert path1.sep.join(["sampledir", "otherfile"]) in l

    def test_visit_norecurse(self, path1):
        l = []
        for i in path1.visit(None, lambda x: x.basename != "sampledir"):
            l.append(i.relto(path1))
        assert "sampledir" in l
        assert not path1.sep.join(["sampledir", "otherfile"]) in l

    @pytest.mark.parametrize('fil', ['*dir', u'*dir',
                             pytest.mark.skip("sys.version_info <"
                                              " (3,6)")(b'*dir')])
    def test_visit_filterfunc_is_string(self, path1, fil):
        l = []
        for i in path1.visit(fil):
            l.append(i.relto(path1))
        assert len(l), 2
        assert "sampledir" in l
        assert "otherdir" in l

    @py.test.mark.xfail("sys.platform.startswith('java')")
    def test_visit_ignore(self, path1):
        p = path1.join('nonexisting')
        assert list(p.visit(ignore=py.error.ENOENT)) == []

    def test_visit_endswith(self, path1):
        l = []
        for i in path1.visit(lambda x: x.check(endswith="file")):
            l.append(i.relto(path1))
        assert path1.sep.join(["sampledir", "otherfile"]) in l
        assert "samplefile" in l

    def test_endswith(self, path1):
        assert path1.check(notendswith='.py')
        x = path1.join('samplefile')
        assert x.check(endswith='file')

    def test_cmp(self, path1):
        path1 = path1.join('samplefile')
        path2 = path1.join('samplefile2')
        assert (path1 < path2) == ('samplefile' < 'samplefile2')
        assert not (path1 < path1)

    def test_simple_read(self, path1):
        x = path1.join('samplefile').read('r')
        assert x == 'samplefile\n'

    def test_join_div_operator(self, path1):
        newpath = path1 / '/sampledir' / '/test//'
        newpath2 = path1.join('sampledir', 'test')
        assert newpath == newpath2

    def test_ext(self, path1):
        newpath = path1.join('sampledir.ext')
        assert newpath.ext == '.ext'
        newpath = path1.join('sampledir')
        assert not newpath.ext

    def test_purebasename(self, path1):
        newpath = path1.join('samplefile.py')
        assert newpath.purebasename == 'samplefile'

    def test_multiple_parts(self, path1):
        newpath = path1.join('samplefile.py')
        dirname, purebasename, basename, ext = newpath._getbyspec(
            'dirname,purebasename,basename,ext')
        assert str(path1).endswith(dirname) # be careful with win32 'drive'
        assert purebasename == 'samplefile'
        assert basename == 'samplefile.py'
        assert ext == '.py'

    def test_dotted_name_ext(self, path1):
        newpath = path1.join('a.b.c')
        ext = newpath.ext
        assert ext == '.c'
        assert newpath.ext == '.c'

    def test_newext(self, path1):
        newpath = path1.join('samplefile.py')
        newext = newpath.new(ext='.txt')
        assert newext.basename == "samplefile.txt"
        assert newext.purebasename == "samplefile"

    def test_readlines(self, path1):
        fn = path1.join('samplefile')
        contents = fn.readlines()
        assert contents == ['samplefile\n']

    def test_readlines_nocr(self, path1):
        fn = path1.join('samplefile')
        contents = fn.readlines(cr=0)
        assert contents == ['samplefile', '']

    def test_file(self, path1):
        assert path1.join('samplefile').check(file=1)

    def test_not_file(self, path1):
        assert not path1.join("sampledir").check(file=1)
        assert path1.join("sampledir").check(file=0)

    def test_non_existent(self, path1):
        assert path1.join("sampledir.nothere").check(dir=0)
        assert path1.join("sampledir.nothere").check(file=0)
        assert path1.join("sampledir.nothere").check(notfile=1)
        assert path1.join("sampledir.nothere").check(notdir=1)
        assert path1.join("sampledir.nothere").check(notexists=1)
        assert not path1.join("sampledir.nothere").check(notfile=0)

    #    pattern = path1.sep.join(['s*file'])
    #    sfile = path1.join("samplefile")
    #    assert sfile.check(fnmatch=pattern)

    def test_size(self, path1):
        url = path1.join("samplefile")
        assert url.size() > len("samplefile")

    def test_mtime(self, path1):
        url = path1.join("samplefile")
        assert url.mtime() > 0

    def test_relto_wrong_type(self, path1):
        py.test.raises(TypeError, "path1.relto(42)")

    def test_load(self, path1):
        p = path1.join('samplepickle')
        obj = p.load()
        assert type(obj) is dict
        assert obj.get('answer',None) == 42

    def test_visit_filesonly(self, path1):
        l = []
        for i in path1.visit(lambda x: x.check(file=1)):
            l.append(i.relto(path1))
        assert not "sampledir" in l
        assert path1.sep.join(["sampledir", "otherfile"]) in l

    def test_visit_nodotfiles(self, path1):
        l = []
        for i in path1.visit(lambda x: x.check(dotfile=0)):
            l.append(i.relto(path1))
        assert "sampledir" in l
        assert path1.sep.join(["sampledir", "otherfile"]) in l
        assert not ".dotfile" in l

    def test_visit_breadthfirst(self, path1):
        l = []
        for i in path1.visit(bf=True):
            l.append(i.relto(path1))
        for i, p in enumerate(l):
            if path1.sep in p:
                for j in range(i, len(l)):
                    assert path1.sep in l[j]
                break
        else:
            py.test.fail("huh")

    def test_visit_sort(self, path1):
        l = []
        for i in path1.visit(bf=True, sort=True):
            l.append(i.relto(path1))
        for i, p in enumerate(l):
            if path1.sep in p:
                break
        assert l[:i] == sorted(l[:i])
        assert l[i:] == sorted(l[i:])

    def test_endswith(self, path1):
        def chk(p):
            return p.check(endswith="pickle")
        assert not chk(path1)
        assert not chk(path1.join('samplefile'))
        assert chk(path1.join('somepickle'))

    def test_copy_file(self, path1):
        otherdir = path1.join('otherdir')
        initpy = otherdir.join('__init__.py')
        copied = otherdir.join('copied')
        initpy.copy(copied)
        try:
            assert copied.check()
            s1 = initpy.read()
            s2 = copied.read()
            assert s1 == s2
        finally:
            if copied.check():
                copied.remove()

    def test_copy_dir(self, path1):
        otherdir = path1.join('otherdir')
        copied = path1.join('newdir')
        try:
            otherdir.copy(copied)
            assert copied.check(dir=1)
            assert copied.join('__init__.py').check(file=1)
            s1 = otherdir.join('__init__.py').read()
            s2 = copied.join('__init__.py').read()
            assert s1 == s2
        finally:
            if copied.check(dir=1):
                copied.remove(rec=1)

    def test_remove_file(self, path1):
        d = path1.ensure('todeleted')
        assert d.check()
        d.remove()
        assert not d.check()

    def test_remove_dir_recursive_by_default(self, path1):
        d = path1.ensure('to', 'be', 'deleted')
        assert d.check()
        p = path1.join('to')
        p.remove()
        assert not p.check()

    def test_ensure_dir(self, path1):
        b = path1.ensure_dir("001", "002")
        assert b.basename == "002"
        assert b.isdir()

    def test_mkdir_and_remove(self, path1):
        tmpdir = path1
        py.test.raises(py.error.EEXIST, tmpdir.mkdir, 'sampledir')
        new = tmpdir.join('mktest1')
        new.mkdir()
        assert new.check(dir=1)
        new.remove()

        new = tmpdir.mkdir('mktest')
        assert new.check(dir=1)
        new.remove()
        assert tmpdir.join('mktest') == new

    def test_move_file(self, path1):
        p = path1.join('samplefile')
        newp = p.dirpath('moved_samplefile')
        p.move(newp)
        try:
            assert newp.check(file=1)
            assert not p.check()
        finally:
            dp = newp.dirpath()
            if hasattr(dp, 'revert'):
                dp.revert()
            else:
                newp.move(p)
                assert p.check()

    def test_move_dir(self, path1):
        source = path1.join('sampledir')
        dest = path1.join('moveddir')
        source.move(dest)
        assert dest.check(dir=1)
        assert dest.join('otherfile').check(file=1)
        assert not source.join('sampledir').check()

    def test_fspath_protocol_match_strpath(self, path1):
        assert path1.__fspath__() == path1.strpath

    def test_fspath_func_match_strpath(self, path1):
        try:
            from os import fspath
        except ImportError:
            from py._path.common import fspath
        assert fspath(path1) == path1.strpath

    @py.test.mark.skip("sys.version_info < (3,6)")
    def test_fspath_open(self, path1):
        f = path1.join('opentestfile')
        open(f)

    @py.test.mark.skip("sys.version_info < (3,6)")
    def test_fspath_fsencode(self, path1):
        from os import fsencode
        assert fsencode(path1) == fsencode(path1.strpath)

def setuptestfs(path):
    if path.join('samplefile').check():
        return
    #print "setting up test fs for", repr(path)
    samplefile = path.ensure('samplefile')
    samplefile.write('samplefile\n')

    execfile = path.ensure('execfile')
    execfile.write('x=42')

    execfilepy = path.ensure('execfile.py')
    execfilepy.write('x=42')

    d = {1:2, 'hello': 'world', 'answer': 42}
    path.ensure('samplepickle').dump(d)

    sampledir = path.ensure('sampledir', dir=1)
    sampledir.ensure('otherfile')

    otherdir = path.ensure('otherdir', dir=1)
    otherdir.ensure('__init__.py')

    module_a = otherdir.ensure('a.py')
    module_a.write('from .b import stuff as result\n')
    module_b = otherdir.ensure('b.py')
    module_b.write('stuff="got it"\n')
    module_c = otherdir.ensure('c.py')
    module_c.write('''import py;
import otherdir.a
value = otherdir.a.result
''')
    module_d = otherdir.ensure('d.py')
    module_d.write('''import py;
from otherdir import a
value2 = a.result
''')
