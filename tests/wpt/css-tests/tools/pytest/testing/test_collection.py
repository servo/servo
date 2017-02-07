import pytest, py

from _pytest.main import Session, EXIT_NOTESTSCOLLECTED

class TestCollector:
    def test_collect_versus_item(self):
        from pytest import Collector, Item
        assert not issubclass(Collector, Item)
        assert not issubclass(Item, Collector)

    def test_compat_attributes(self, testdir, recwarn):
        modcol = testdir.getmodulecol("""
            def test_pass(): pass
            def test_fail(): assert 0
        """)
        recwarn.clear()
        assert modcol.Module == pytest.Module
        assert modcol.Class == pytest.Class
        assert modcol.Item == pytest.Item
        assert modcol.File == pytest.File
        assert modcol.Function == pytest.Function

    def test_check_equality(self, testdir):
        modcol = testdir.getmodulecol("""
            def test_pass(): pass
            def test_fail(): assert 0
        """)
        fn1 = testdir.collect_by_name(modcol, "test_pass")
        assert isinstance(fn1, pytest.Function)
        fn2 = testdir.collect_by_name(modcol, "test_pass")
        assert isinstance(fn2, pytest.Function)

        assert fn1 == fn2
        assert fn1 != modcol
        if py.std.sys.version_info < (3, 0):
            assert cmp(fn1, fn2) == 0
        assert hash(fn1) == hash(fn2)

        fn3 = testdir.collect_by_name(modcol, "test_fail")
        assert isinstance(fn3, pytest.Function)
        assert not (fn1 == fn3)
        assert fn1 != fn3

        for fn in fn1,fn2,fn3:
            assert fn != 3
            assert fn != modcol
            assert fn != [1,2,3]
            assert [1,2,3] != fn
            assert modcol != fn

    def test_getparent(self, testdir):
        modcol = testdir.getmodulecol("""
            class TestClass:
                 def test_foo():
                     pass
        """)
        cls = testdir.collect_by_name(modcol, "TestClass")
        fn = testdir.collect_by_name(
            testdir.collect_by_name(cls, "()"), "test_foo")

        parent = fn.getparent(pytest.Module)
        assert parent is modcol

        parent = fn.getparent(pytest.Function)
        assert parent is fn

        parent = fn.getparent(pytest.Class)
        assert parent is cls


    def test_getcustomfile_roundtrip(self, testdir):
        hello = testdir.makefile(".xxx", hello="world")
        testdir.makepyfile(conftest="""
            import pytest
            class CustomFile(pytest.File):
                pass
            def pytest_collect_file(path, parent):
                if path.ext == ".xxx":
                    return CustomFile(path, parent=parent)
        """)
        node = testdir.getpathnode(hello)
        assert isinstance(node, pytest.File)
        assert node.name == "hello.xxx"
        nodes = node.session.perform_collect([node.nodeid], genitems=False)
        assert len(nodes) == 1
        assert isinstance(nodes[0], pytest.File)

class TestCollectFS:
    def test_ignored_certain_directories(self, testdir):
        tmpdir = testdir.tmpdir
        tmpdir.ensure("_darcs", 'test_notfound.py')
        tmpdir.ensure("CVS", 'test_notfound.py')
        tmpdir.ensure("{arch}", 'test_notfound.py')
        tmpdir.ensure(".whatever", 'test_notfound.py')
        tmpdir.ensure(".bzr", 'test_notfound.py')
        tmpdir.ensure("normal", 'test_found.py')
        for x in tmpdir.visit("test_*.py"):
            x.write("def test_hello(): pass")

        result = testdir.runpytest("--collect-only")
        s = result.stdout.str()
        assert "test_notfound" not in s
        assert "test_found" in s

    def test_custom_norecursedirs(self, testdir):
        testdir.makeini("""
            [pytest]
            norecursedirs = mydir xyz*
        """)
        tmpdir = testdir.tmpdir
        tmpdir.ensure("mydir", "test_hello.py").write("def test_1(): pass")
        tmpdir.ensure("xyz123", "test_2.py").write("def test_2(): 0/0")
        tmpdir.ensure("xy", "test_ok.py").write("def test_3(): pass")
        rec = testdir.inline_run()
        rec.assertoutcome(passed=1)
        rec = testdir.inline_run("xyz123/test_2.py")
        rec.assertoutcome(failed=1)

    def test_testpaths_ini(self, testdir, monkeypatch):
        testdir.makeini("""
            [pytest]
            testpaths = gui uts
        """)
        tmpdir = testdir.tmpdir
        tmpdir.ensure("env", "test_1.py").write("def test_env(): pass")
        tmpdir.ensure("gui", "test_2.py").write("def test_gui(): pass")
        tmpdir.ensure("uts", "test_3.py").write("def test_uts(): pass")

        # executing from rootdir only tests from `testpaths` directories
        # are collected
        items, reprec = testdir.inline_genitems('-v')
        assert [x.name for x in items] == ['test_gui', 'test_uts']

        # check that explicitly passing directories in the command-line
        # collects the tests
        for dirname in ('env', 'gui', 'uts'):
            items, reprec = testdir.inline_genitems(tmpdir.join(dirname))
            assert [x.name for x in items] == ['test_%s' % dirname]

        # changing cwd to each subdirectory and running pytest without
        # arguments collects the tests in that directory normally
        for dirname in ('env', 'gui', 'uts'):
            monkeypatch.chdir(testdir.tmpdir.join(dirname))
            items, reprec = testdir.inline_genitems()
            assert [x.name for x in items] == ['test_%s' % dirname]


class TestCollectPluginHookRelay:
    def test_pytest_collect_file(self, testdir):
        wascalled = []
        class Plugin:
            def pytest_collect_file(self, path, parent):
                wascalled.append(path)
        testdir.makefile(".abc", "xyz")
        pytest.main([testdir.tmpdir], plugins=[Plugin()])
        assert len(wascalled) == 1
        assert wascalled[0].ext == '.abc'

    def test_pytest_collect_directory(self, testdir):
        wascalled = []
        class Plugin:
            def pytest_collect_directory(self, path, parent):
                wascalled.append(path.basename)
        testdir.mkdir("hello")
        testdir.mkdir("world")
        pytest.main(testdir.tmpdir, plugins=[Plugin()])
        assert "hello" in wascalled
        assert "world" in wascalled

class TestPrunetraceback:
    def test_collection_error(self, testdir):
        p = testdir.makepyfile("""
            import not_exists
        """)
        result = testdir.runpytest(p)
        assert "__import__" not in result.stdout.str(), "too long traceback"
        result.stdout.fnmatch_lines([
            "*ERROR collecting*",
            "*mport*not_exists*"
        ])

    def test_custom_repr_failure(self, testdir):
        p = testdir.makepyfile("""
            import not_exists
        """)
        testdir.makeconftest("""
            import pytest
            def pytest_collect_file(path, parent):
                return MyFile(path, parent)
            class MyError(Exception):
                pass
            class MyFile(pytest.File):
                def collect(self):
                    raise MyError()
                def repr_failure(self, excinfo):
                    if excinfo.errisinstance(MyError):
                        return "hello world"
                    return pytest.File.repr_failure(self, excinfo)
        """)

        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines([
            "*ERROR collecting*",
            "*hello world*",
        ])

    @pytest.mark.xfail(reason="other mechanism for adding to reporting needed")
    def test_collect_report_postprocessing(self, testdir):
        p = testdir.makepyfile("""
            import not_exists
        """)
        testdir.makeconftest("""
            import pytest
            def pytest_make_collect_report(__multicall__):
                rep = __multicall__.execute()
                rep.headerlines += ["header1"]
                return rep
        """)
        result = testdir.runpytest(p)
        result.stdout.fnmatch_lines([
            "*ERROR collecting*",
            "*header1*",
        ])


class TestCustomConftests:
    def test_ignore_collect_path(self, testdir):
        testdir.makeconftest("""
            def pytest_ignore_collect(path, config):
                return path.basename.startswith("x") or \
                       path.basename == "test_one.py"
        """)
        sub = testdir.mkdir("xy123")
        sub.ensure("test_hello.py").write("syntax error")
        sub.join("conftest.py").write("syntax error")
        testdir.makepyfile("def test_hello(): pass")
        testdir.makepyfile(test_one="syntax error")
        result = testdir.runpytest("--fulltrace")
        assert result.ret == 0
        result.stdout.fnmatch_lines(["*1 passed*"])

    def test_ignore_collect_not_called_on_argument(self, testdir):
        testdir.makeconftest("""
            def pytest_ignore_collect(path, config):
                return True
        """)
        p = testdir.makepyfile("def test_hello(): pass")
        result = testdir.runpytest(p)
        assert result.ret == 0
        result.stdout.fnmatch_lines("*1 passed*")
        result = testdir.runpytest()
        assert result.ret == EXIT_NOTESTSCOLLECTED
        result.stdout.fnmatch_lines("*collected 0 items*")

    def test_collectignore_exclude_on_option(self, testdir):
        testdir.makeconftest("""
            collect_ignore = ['hello', 'test_world.py']
            def pytest_addoption(parser):
                parser.addoption("--XX", action="store_true", default=False)
            def pytest_configure(config):
                if config.getvalue("XX"):
                    collect_ignore[:] = []
        """)
        testdir.mkdir("hello")
        testdir.makepyfile(test_world="def test_hello(): pass")
        result = testdir.runpytest()
        assert result.ret == EXIT_NOTESTSCOLLECTED
        assert "passed" not in result.stdout.str()
        result = testdir.runpytest("--XX")
        assert result.ret == 0
        assert "passed" in result.stdout.str()

    def test_pytest_fs_collect_hooks_are_seen(self, testdir):
        testdir.makeconftest("""
            import pytest
            class MyModule(pytest.Module):
                pass
            def pytest_collect_file(path, parent):
                if path.ext == ".py":
                    return MyModule(path, parent)
        """)
        testdir.mkdir("sub")
        testdir.makepyfile("def test_x(): pass")
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines([
            "*MyModule*",
            "*test_x*"
        ])

    def test_pytest_collect_file_from_sister_dir(self, testdir):
        sub1 = testdir.mkpydir("sub1")
        sub2 = testdir.mkpydir("sub2")
        conf1 = testdir.makeconftest("""
            import pytest
            class MyModule1(pytest.Module):
                pass
            def pytest_collect_file(path, parent):
                if path.ext == ".py":
                    return MyModule1(path, parent)
        """)
        conf1.move(sub1.join(conf1.basename))
        conf2 = testdir.makeconftest("""
            import pytest
            class MyModule2(pytest.Module):
                pass
            def pytest_collect_file(path, parent):
                if path.ext == ".py":
                    return MyModule2(path, parent)
        """)
        conf2.move(sub2.join(conf2.basename))
        p = testdir.makepyfile("def test_x(): pass")
        p.copy(sub1.join(p.basename))
        p.copy(sub2.join(p.basename))
        result = testdir.runpytest("--collect-only")
        result.stdout.fnmatch_lines([
            "*MyModule1*",
            "*MyModule2*",
            "*test_x*"
        ])

class TestSession:
    def test_parsearg(self, testdir):
        p = testdir.makepyfile("def test_func(): pass")
        subdir = testdir.mkdir("sub")
        subdir.ensure("__init__.py")
        target = subdir.join(p.basename)
        p.move(target)
        subdir.chdir()
        config = testdir.parseconfig(p.basename)
        rcol = Session(config=config)
        assert rcol.fspath == subdir
        parts = rcol._parsearg(p.basename)

        assert parts[0] ==  target
        assert len(parts) == 1
        parts = rcol._parsearg(p.basename + "::test_func")
        assert parts[0] ==  target
        assert parts[1] ==  "test_func"
        assert len(parts) == 2

    def test_collect_topdir(self, testdir):
        p = testdir.makepyfile("def test_func(): pass")
        id = "::".join([p.basename, "test_func"])
        # XXX migrate to collectonly? (see below)
        config = testdir.parseconfig(id)
        topdir = testdir.tmpdir
        rcol = Session(config)
        assert topdir == rcol.fspath
        #rootid = rcol.nodeid
        #root2 = rcol.perform_collect([rcol.nodeid], genitems=False)[0]
        #assert root2 == rcol, rootid
        colitems = rcol.perform_collect([rcol.nodeid], genitems=False)
        assert len(colitems) == 1
        assert colitems[0].fspath == p


    def test_collect_protocol_single_function(self, testdir):
        p = testdir.makepyfile("def test_func(): pass")
        id = "::".join([p.basename, "test_func"])
        items, hookrec = testdir.inline_genitems(id)
        item, = items
        assert item.name == "test_func"
        newid = item.nodeid
        assert newid == id
        py.std.pprint.pprint(hookrec.calls)
        topdir = testdir.tmpdir  # noqa
        hookrec.assert_contains([
            ("pytest_collectstart", "collector.fspath == topdir"),
            ("pytest_make_collect_report", "collector.fspath == topdir"),
            ("pytest_collectstart", "collector.fspath == p"),
            ("pytest_make_collect_report", "collector.fspath == p"),
            ("pytest_pycollect_makeitem", "name == 'test_func'"),
            ("pytest_collectreport", "report.nodeid.startswith(p.basename)"),
            ("pytest_collectreport", "report.nodeid == ''")
        ])

    def test_collect_protocol_method(self, testdir):
        p = testdir.makepyfile("""
            class TestClass:
                def test_method(self):
                    pass
        """)
        normid = p.basename + "::TestClass::()::test_method"
        for id in [p.basename,
                   p.basename + "::TestClass",
                   p.basename + "::TestClass::()",
                   normid,
                   ]:
            items, hookrec = testdir.inline_genitems(id)
            assert len(items) == 1
            assert items[0].name == "test_method"
            newid = items[0].nodeid
            assert newid == normid

    def test_collect_custom_nodes_multi_id(self, testdir):
        p = testdir.makepyfile("def test_func(): pass")
        testdir.makeconftest("""
            import pytest
            class SpecialItem(pytest.Item):
                def runtest(self):
                    return # ok
            class SpecialFile(pytest.File):
                def collect(self):
                    return [SpecialItem(name="check", parent=self)]
            def pytest_collect_file(path, parent):
                if path.basename == %r:
                    return SpecialFile(fspath=path, parent=parent)
        """ % p.basename)
        id = p.basename

        items, hookrec = testdir.inline_genitems(id)
        py.std.pprint.pprint(hookrec.calls)
        assert len(items) == 2
        hookrec.assert_contains([
            ("pytest_collectstart",
                "collector.fspath == collector.session.fspath"),
            ("pytest_collectstart",
                "collector.__class__.__name__ == 'SpecialFile'"),
            ("pytest_collectstart",
                "collector.__class__.__name__ == 'Module'"),
            ("pytest_pycollect_makeitem", "name == 'test_func'"),
            ("pytest_collectreport", "report.nodeid.startswith(p.basename)"),
            #("pytest_collectreport",
            #    "report.fspath == %r" % str(rcol.fspath)),
        ])

    def test_collect_subdir_event_ordering(self, testdir):
        p = testdir.makepyfile("def test_func(): pass")
        aaa = testdir.mkpydir("aaa")
        test_aaa = aaa.join("test_aaa.py")
        p.move(test_aaa)

        items, hookrec = testdir.inline_genitems()
        assert len(items) == 1
        py.std.pprint.pprint(hookrec.calls)
        hookrec.assert_contains([
            ("pytest_collectstart", "collector.fspath == test_aaa"),
            ("pytest_pycollect_makeitem", "name == 'test_func'"),
            ("pytest_collectreport",
                    "report.nodeid.startswith('aaa/test_aaa.py')"),
        ])

    def test_collect_two_commandline_args(self, testdir):
        p = testdir.makepyfile("def test_func(): pass")
        aaa = testdir.mkpydir("aaa")
        bbb = testdir.mkpydir("bbb")
        test_aaa = aaa.join("test_aaa.py")
        p.copy(test_aaa)
        test_bbb = bbb.join("test_bbb.py")
        p.move(test_bbb)

        id = "."

        items, hookrec = testdir.inline_genitems(id)
        assert len(items) == 2
        py.std.pprint.pprint(hookrec.calls)
        hookrec.assert_contains([
            ("pytest_collectstart", "collector.fspath == test_aaa"),
            ("pytest_pycollect_makeitem", "name == 'test_func'"),
            ("pytest_collectreport", "report.nodeid == 'aaa/test_aaa.py'"),
            ("pytest_collectstart", "collector.fspath == test_bbb"),
            ("pytest_pycollect_makeitem", "name == 'test_func'"),
            ("pytest_collectreport", "report.nodeid == 'bbb/test_bbb.py'"),
        ])

    def test_serialization_byid(self, testdir):
        testdir.makepyfile("def test_func(): pass")
        items, hookrec = testdir.inline_genitems()
        assert len(items) == 1
        item, = items
        items2, hookrec = testdir.inline_genitems(item.nodeid)
        item2, = items2
        assert item2.name == item.name
        assert item2.fspath == item.fspath

    def test_find_byid_without_instance_parents(self, testdir):
        p = testdir.makepyfile("""
            class TestClass:
                def test_method(self):
                    pass
        """)
        arg = p.basename + ("::TestClass::test_method")
        items, hookrec = testdir.inline_genitems(arg)
        assert len(items) == 1
        item, = items
        assert item.nodeid.endswith("TestClass::()::test_method")

class Test_getinitialnodes:
    def test_global_file(self, testdir, tmpdir):
        x = tmpdir.ensure("x.py")
        config = testdir.parseconfigure(x)
        col = testdir.getnode(config, x)
        assert isinstance(col, pytest.Module)
        assert col.name == 'x.py'
        assert col.parent.name == testdir.tmpdir.basename
        assert col.parent.parent is None
        for col in col.listchain():
            assert col.config is config

    def test_pkgfile(self, testdir):
        tmpdir = testdir.tmpdir
        subdir = tmpdir.join("subdir")
        x = subdir.ensure("x.py")
        subdir.ensure("__init__.py")
        config = testdir.parseconfigure(x)
        col = testdir.getnode(config, x)
        assert isinstance(col, pytest.Module)
        assert col.name == 'x.py'
        assert col.parent.parent is None
        for col in col.listchain():
            assert col.config is config

class Test_genitems:
    def test_check_collect_hashes(self, testdir):
        p = testdir.makepyfile("""
            def test_1():
                pass

            def test_2():
                pass
        """)
        p.copy(p.dirpath(p.purebasename + "2" + ".py"))
        items, reprec = testdir.inline_genitems(p.dirpath())
        assert len(items) == 4
        for numi, i in enumerate(items):
            for numj, j in enumerate(items):
                if numj != numi:
                    assert hash(i) != hash(j)
                    assert i != j

    def test_example_items1(self, testdir):
        p = testdir.makepyfile('''
            def testone():
                pass

            class TestX:
                def testmethod_one(self):
                    pass

            class TestY(TestX):
                pass
        ''')
        items, reprec = testdir.inline_genitems(p)
        assert len(items) == 3
        assert items[0].name == 'testone'
        assert items[1].name == 'testmethod_one'
        assert items[2].name == 'testmethod_one'

        # let's also test getmodpath here
        assert items[0].getmodpath() == "testone"
        assert items[1].getmodpath() == "TestX.testmethod_one"
        assert items[2].getmodpath() == "TestY.testmethod_one"

        s = items[0].getmodpath(stopatmodule=False)
        assert s.endswith("test_example_items1.testone")
        print(s)

    def test_class_and_functions_discovery_using_glob(self, testdir):
        """
        tests that python_classes and python_functions config options work
        as prefixes and glob-like patterns (issue #600).
        """
        testdir.makeini("""
            [pytest]
            python_classes = *Suite Test
            python_functions = *_test test
        """)
        p = testdir.makepyfile('''
            class MyTestSuite:
                def x_test(self):
                    pass

            class TestCase:
                def test_y(self):
                    pass
        ''')
        items, reprec = testdir.inline_genitems(p)
        ids = [x.getmodpath() for x in items]
        assert ids == ['MyTestSuite.x_test', 'TestCase.test_y']


def test_matchnodes_two_collections_same_file(testdir):
    testdir.makeconftest("""
        import pytest
        def pytest_configure(config):
            config.pluginmanager.register(Plugin2())

        class Plugin2:
            def pytest_collect_file(self, path, parent):
                if path.ext == ".abc":
                    return MyFile2(path, parent)

        def pytest_collect_file(path, parent):
            if path.ext == ".abc":
                return MyFile1(path, parent)

        class MyFile1(pytest.Item, pytest.File):
            def runtest(self):
                pass
        class MyFile2(pytest.File):
            def collect(self):
                return [Item2("hello", parent=self)]

        class Item2(pytest.Item):
            def runtest(self):
                pass
    """)
    p = testdir.makefile(".abc", "")
    result = testdir.runpytest()
    assert result.ret == 0
    result.stdout.fnmatch_lines([
        "*2 passed*",
    ])
    res = testdir.runpytest("%s::hello" % p.basename)
    res.stdout.fnmatch_lines([
        "*1 passed*",
    ])


class TestNodekeywords:
    def test_no_under(self, testdir):
        modcol = testdir.getmodulecol("""
            def test_pass(): pass
            def test_fail(): assert 0
        """)
        l = list(modcol.keywords)
        assert modcol.name in l
        for x in l:
            assert not x.startswith("_")
        assert modcol.name in repr(modcol.keywords)

    def test_issue345(self, testdir):
        testdir.makepyfile("""
            def test_should_not_be_selected():
                assert False, 'I should not have been selected to run'

            def test___repr__():
                pass
        """)
        reprec = testdir.inline_run("-k repr")
        reprec.assertoutcome(passed=1, failed=0)
