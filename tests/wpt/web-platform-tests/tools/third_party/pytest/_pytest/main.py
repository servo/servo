""" core implementation of testing process: init, session, runtest loop. """
from __future__ import absolute_import, division, print_function

import functools
import os
import six
import sys

import _pytest
from _pytest import nodes
import _pytest._code
import py
try:
    from collections import MutableMapping as MappingMixin
except ImportError:
    from UserDict import DictMixin as MappingMixin

from _pytest.config import directory_arg, UsageError, hookimpl
from _pytest.outcomes import exit
from _pytest.runner import collect_one_node

tracebackcutdir = py.path.local(_pytest.__file__).dirpath()

# exitcodes for the command line
EXIT_OK = 0
EXIT_TESTSFAILED = 1
EXIT_INTERRUPTED = 2
EXIT_INTERNALERROR = 3
EXIT_USAGEERROR = 4
EXIT_NOTESTSCOLLECTED = 5


def pytest_addoption(parser):
    parser.addini("norecursedirs", "directory patterns to avoid for recursion",
                  type="args", default=['.*', 'build', 'dist', 'CVS', '_darcs', '{arch}', '*.egg', 'venv'])
    parser.addini("testpaths", "directories to search for tests when no files or directories are given in the "
                               "command line.",
                  type="args", default=[])
    # parser.addini("dirpatterns",
    #    "patterns specifying possible locations of test files",
    #    type="linelist", default=["**/test_*.txt",
    #            "**/test_*.py", "**/*_test.py"]
    # )
    group = parser.getgroup("general", "running and selection options")
    group._addoption('-x', '--exitfirst', action="store_const",
                     dest="maxfail", const=1,
                     help="exit instantly on first error or failed test."),
    group._addoption('--maxfail', metavar="num",
                     action="store", type=int, dest="maxfail", default=0,
                     help="exit after first num failures or errors.")
    group._addoption('--strict', action="store_true",
                     help="marks not registered in configuration file raise errors.")
    group._addoption("-c", metavar="file", type=str, dest="inifilename",
                     help="load configuration from `file` instead of trying to locate one of the implicit "
                          "configuration files.")
    group._addoption("--continue-on-collection-errors", action="store_true",
                     default=False, dest="continue_on_collection_errors",
                     help="Force test execution even if collection errors occur.")

    group = parser.getgroup("collect", "collection")
    group.addoption('--collectonly', '--collect-only', action="store_true",
                    help="only collect tests, don't execute them."),
    group.addoption('--pyargs', action="store_true",
                    help="try to interpret all arguments as python packages.")
    group.addoption("--ignore", action="append", metavar="path",
                    help="ignore path during collection (multi-allowed).")
    # when changing this to --conf-cut-dir, config.py Conftest.setinitial
    # needs upgrading as well
    group.addoption('--confcutdir', dest="confcutdir", default=None,
                    metavar="dir", type=functools.partial(directory_arg, optname="--confcutdir"),
                    help="only load conftest.py's relative to specified dir.")
    group.addoption('--noconftest', action="store_true",
                    dest="noconftest", default=False,
                    help="Don't load any conftest.py files.")
    group.addoption('--keepduplicates', '--keep-duplicates', action="store_true",
                    dest="keepduplicates", default=False,
                    help="Keep duplicate tests.")
    group.addoption('--collect-in-virtualenv', action='store_true',
                    dest='collect_in_virtualenv', default=False,
                    help="Don't ignore tests in a local virtualenv directory")

    group = parser.getgroup("debugconfig",
                            "test session debugging and configuration")
    group.addoption('--basetemp', dest="basetemp", default=None, metavar="dir",
                    help="base temporary directory for this test run.")


def pytest_configure(config):
    __import__('pytest').config = config  # compatibiltiy


def wrap_session(config, doit):
    """Skeleton command line program"""
    session = Session(config)
    session.exitstatus = EXIT_OK
    initstate = 0
    try:
        try:
            config._do_configure()
            initstate = 1
            config.hook.pytest_sessionstart(session=session)
            initstate = 2
            session.exitstatus = doit(config, session) or 0
        except UsageError:
            raise
        except Failed:
            session.exitstatus = EXIT_TESTSFAILED
        except KeyboardInterrupt:
            excinfo = _pytest._code.ExceptionInfo()
            if initstate < 2 and isinstance(excinfo.value, exit.Exception):
                sys.stderr.write('{0}: {1}\n'.format(
                    excinfo.typename, excinfo.value.msg))
            config.hook.pytest_keyboard_interrupt(excinfo=excinfo)
            session.exitstatus = EXIT_INTERRUPTED
        except:  # noqa
            excinfo = _pytest._code.ExceptionInfo()
            config.notify_exception(excinfo, config.option)
            session.exitstatus = EXIT_INTERNALERROR
            if excinfo.errisinstance(SystemExit):
                sys.stderr.write("mainloop: caught Spurious SystemExit!\n")

    finally:
        excinfo = None  # Explicitly break reference cycle.
        session.startdir.chdir()
        if initstate >= 2:
            config.hook.pytest_sessionfinish(
                session=session,
                exitstatus=session.exitstatus)
        config._ensure_unconfigure()
    return session.exitstatus


def pytest_cmdline_main(config):
    return wrap_session(config, _main)


def _main(config, session):
    """ default command line protocol for initialization, session,
    running tests and reporting. """
    config.hook.pytest_collection(session=session)
    config.hook.pytest_runtestloop(session=session)

    if session.testsfailed:
        return EXIT_TESTSFAILED
    elif session.testscollected == 0:
        return EXIT_NOTESTSCOLLECTED


def pytest_collection(session):
    return session.perform_collect()


def pytest_runtestloop(session):
    if (session.testsfailed and
            not session.config.option.continue_on_collection_errors):
        raise session.Interrupted(
            "%d errors during collection" % session.testsfailed)

    if session.config.option.collectonly:
        return True

    for i, item in enumerate(session.items):
        nextitem = session.items[i + 1] if i + 1 < len(session.items) else None
        item.config.hook.pytest_runtest_protocol(item=item, nextitem=nextitem)
        if session.shouldfail:
            raise session.Failed(session.shouldfail)
        if session.shouldstop:
            raise session.Interrupted(session.shouldstop)
    return True


def _in_venv(path):
    """Attempts to detect if ``path`` is the root of a Virtual Environment by
    checking for the existence of the appropriate activate script"""
    bindir = path.join('Scripts' if sys.platform.startswith('win') else 'bin')
    if not bindir.exists():
        return False
    activates = ('activate', 'activate.csh', 'activate.fish',
                 'Activate', 'Activate.bat', 'Activate.ps1')
    return any([fname.basename in activates for fname in bindir.listdir()])


def pytest_ignore_collect(path, config):
    ignore_paths = config._getconftest_pathlist("collect_ignore", path=path.dirpath())
    ignore_paths = ignore_paths or []
    excludeopt = config.getoption("ignore")
    if excludeopt:
        ignore_paths.extend([py.path.local(x) for x in excludeopt])

    if py.path.local(path) in ignore_paths:
        return True

    allow_in_venv = config.getoption("collect_in_virtualenv")
    if _in_venv(path) and not allow_in_venv:
        return True

    # Skip duplicate paths.
    keepduplicates = config.getoption("keepduplicates")
    duplicate_paths = config.pluginmanager._duplicatepaths
    if not keepduplicates:
        if path in duplicate_paths:
            return True
        else:
            duplicate_paths.add(path)

    return False


class FSHookProxy:
    def __init__(self, fspath, pm, remove_mods):
        self.fspath = fspath
        self.pm = pm
        self.remove_mods = remove_mods

    def __getattr__(self, name):
        x = self.pm.subset_hook_caller(name, remove_plugins=self.remove_mods)
        self.__dict__[name] = x
        return x


class _CompatProperty(object):
    def __init__(self, name):
        self.name = name

    def __get__(self, obj, owner):
        if obj is None:
            return self

        # TODO: reenable in the features branch
        # warnings.warn(
        #     "usage of {owner!r}.{name} is deprecated, please use pytest.{name} instead".format(
        #         name=self.name, owner=type(owner).__name__),
        #     PendingDeprecationWarning, stacklevel=2)
        return getattr(__import__('pytest'), self.name)


class NodeKeywords(MappingMixin):
    def __init__(self, node):
        self.node = node
        self.parent = node.parent
        self._markers = {node.name: True}

    def __getitem__(self, key):
        try:
            return self._markers[key]
        except KeyError:
            if self.parent is None:
                raise
            return self.parent.keywords[key]

    def __setitem__(self, key, value):
        self._markers[key] = value

    def __delitem__(self, key):
        raise ValueError("cannot delete key in keywords dict")

    def __iter__(self):
        seen = set(self._markers)
        if self.parent is not None:
            seen.update(self.parent.keywords)
        return iter(seen)

    def __len__(self):
        return len(self.__iter__())

    def keys(self):
        return list(self)

    def __repr__(self):
        return "<NodeKeywords for node %s>" % (self.node, )


class Node(object):
    """ base class for Collector and Item the test collection tree.
    Collector subclasses have children, Items are terminal nodes."""

    def __init__(self, name, parent=None, config=None, session=None):
        #: a unique name within the scope of the parent node
        self.name = name

        #: the parent collector node.
        self.parent = parent

        #: the pytest config object
        self.config = config or parent.config

        #: the session this node is part of
        self.session = session or parent.session

        #: filesystem path where this node was collected from (can be None)
        self.fspath = getattr(parent, 'fspath', None)

        #: keywords/markers collected from all scopes
        self.keywords = NodeKeywords(self)

        #: allow adding of extra keywords to use for matching
        self.extra_keyword_matches = set()

        # used for storing artificial fixturedefs for direct parametrization
        self._name2pseudofixturedef = {}

    @property
    def ihook(self):
        """ fspath sensitive hook proxy used to call pytest hooks"""
        return self.session.gethookproxy(self.fspath)

    Module = _CompatProperty("Module")
    Class = _CompatProperty("Class")
    Instance = _CompatProperty("Instance")
    Function = _CompatProperty("Function")
    File = _CompatProperty("File")
    Item = _CompatProperty("Item")

    def _getcustomclass(self, name):
        maybe_compatprop = getattr(type(self), name)
        if isinstance(maybe_compatprop, _CompatProperty):
            return getattr(__import__('pytest'), name)
        else:
            cls = getattr(self, name)
            # TODO: reenable in the features branch
            # warnings.warn("use of node.%s is deprecated, "
            #    "use pytest_pycollect_makeitem(...) to create custom "
            #    "collection nodes" % name, category=DeprecationWarning)
        return cls

    def __repr__(self):
        return "<%s %r>" % (self.__class__.__name__,
                            getattr(self, 'name', None))

    def warn(self, code, message):
        """ generate a warning with the given code and message for this
        item. """
        assert isinstance(code, str)
        fslocation = getattr(self, "location", None)
        if fslocation is None:
            fslocation = getattr(self, "fspath", None)
        self.ihook.pytest_logwarning.call_historic(kwargs=dict(
            code=code, message=message,
            nodeid=self.nodeid, fslocation=fslocation))

    # methods for ordering nodes
    @property
    def nodeid(self):
        """ a ::-separated string denoting its collection tree address. """
        try:
            return self._nodeid
        except AttributeError:
            self._nodeid = x = self._makeid()
            return x

    def _makeid(self):
        return self.parent.nodeid + "::" + self.name

    def __hash__(self):
        return hash(self.nodeid)

    def setup(self):
        pass

    def teardown(self):
        pass

    def listchain(self):
        """ return list of all parent collectors up to self,
            starting from root of collection tree. """
        chain = []
        item = self
        while item is not None:
            chain.append(item)
            item = item.parent
        chain.reverse()
        return chain

    def add_marker(self, marker):
        """ dynamically add a marker object to the node.

        ``marker`` can be a string or pytest.mark.* instance.
        """
        from _pytest.mark import MarkDecorator, MARK_GEN
        if isinstance(marker, six.string_types):
            marker = getattr(MARK_GEN, marker)
        elif not isinstance(marker, MarkDecorator):
            raise ValueError("is not a string or pytest.mark.* Marker")
        self.keywords[marker.name] = marker

    def get_marker(self, name):
        """ get a marker object from this node or None if
        the node doesn't have a marker with that name. """
        val = self.keywords.get(name, None)
        if val is not None:
            from _pytest.mark import MarkInfo, MarkDecorator
            if isinstance(val, (MarkDecorator, MarkInfo)):
                return val

    def listextrakeywords(self):
        """ Return a set of all extra keywords in self and any parents."""
        extra_keywords = set()
        item = self
        for item in self.listchain():
            extra_keywords.update(item.extra_keyword_matches)
        return extra_keywords

    def listnames(self):
        return [x.name for x in self.listchain()]

    def addfinalizer(self, fin):
        """ register a function to be called when this node is finalized.

        This method can only be called when this node is active
        in a setup chain, for example during self.setup().
        """
        self.session._setupstate.addfinalizer(fin, self)

    def getparent(self, cls):
        """ get the next parent node (including ourself)
        which is an instance of the given class"""
        current = self
        while current and not isinstance(current, cls):
            current = current.parent
        return current

    def _prunetraceback(self, excinfo):
        pass

    def _repr_failure_py(self, excinfo, style=None):
        fm = self.session._fixturemanager
        if excinfo.errisinstance(fm.FixtureLookupError):
            return excinfo.value.formatrepr()
        tbfilter = True
        if self.config.option.fulltrace:
            style = "long"
        else:
            tb = _pytest._code.Traceback([excinfo.traceback[-1]])
            self._prunetraceback(excinfo)
            if len(excinfo.traceback) == 0:
                excinfo.traceback = tb
            tbfilter = False  # prunetraceback already does it
            if style == "auto":
                style = "long"
        # XXX should excinfo.getrepr record all data and toterminal() process it?
        if style is None:
            if self.config.option.tbstyle == "short":
                style = "short"
            else:
                style = "long"

        try:
            os.getcwd()
            abspath = False
        except OSError:
            abspath = True

        return excinfo.getrepr(funcargs=True, abspath=abspath,
                               showlocals=self.config.option.showlocals,
                               style=style, tbfilter=tbfilter)

    repr_failure = _repr_failure_py


class Collector(Node):
    """ Collector instances create children through collect()
        and thus iteratively build a tree.
    """

    class CollectError(Exception):
        """ an error during collection, contains a custom message. """

    def collect(self):
        """ returns a list of children (items and collectors)
            for this collection node.
        """
        raise NotImplementedError("abstract")

    def repr_failure(self, excinfo):
        """ represent a collection failure. """
        if excinfo.errisinstance(self.CollectError):
            exc = excinfo.value
            return str(exc.args[0])
        return self._repr_failure_py(excinfo, style="short")

    def _prunetraceback(self, excinfo):
        if hasattr(self, 'fspath'):
            traceback = excinfo.traceback
            ntraceback = traceback.cut(path=self.fspath)
            if ntraceback == traceback:
                ntraceback = ntraceback.cut(excludepath=tracebackcutdir)
            excinfo.traceback = ntraceback.filter()


class FSCollector(Collector):
    def __init__(self, fspath, parent=None, config=None, session=None):
        fspath = py.path.local(fspath)  # xxx only for test_resultlog.py?
        name = fspath.basename
        if parent is not None:
            rel = fspath.relto(parent.fspath)
            if rel:
                name = rel
            name = name.replace(os.sep, nodes.SEP)
        super(FSCollector, self).__init__(name, parent, config, session)
        self.fspath = fspath

    def _check_initialpaths_for_relpath(self):
        for initialpath in self.session._initialpaths:
            if self.fspath.common(initialpath) == initialpath:
                return self.fspath.relto(initialpath.dirname)

    def _makeid(self):
        relpath = self.fspath.relto(self.config.rootdir)

        if not relpath:
            relpath = self._check_initialpaths_for_relpath()
        if os.sep != nodes.SEP:
            relpath = relpath.replace(os.sep, nodes.SEP)
        return relpath


class File(FSCollector):
    """ base class for collecting tests from a file. """


class Item(Node):
    """ a basic test invocation item. Note that for a single function
    there might be multiple test invocation items.
    """
    nextitem = None

    def __init__(self, name, parent=None, config=None, session=None):
        super(Item, self).__init__(name, parent, config, session)
        self._report_sections = []

    def add_report_section(self, when, key, content):
        """
        Adds a new report section, similar to what's done internally to add stdout and
        stderr captured output::

            item.add_report_section("call", "stdout", "report section contents")

        :param str when:
            One of the possible capture states, ``"setup"``, ``"call"``, ``"teardown"``.
        :param str key:
            Name of the section, can be customized at will. Pytest uses ``"stdout"`` and
            ``"stderr"`` internally.

        :param str content:
            The full contents as a string.
        """
        if content:
            self._report_sections.append((when, key, content))

    def reportinfo(self):
        return self.fspath, None, ""

    @property
    def location(self):
        try:
            return self._location
        except AttributeError:
            location = self.reportinfo()
            # bestrelpath is a quite slow function
            cache = self.config.__dict__.setdefault("_bestrelpathcache", {})
            try:
                fspath = cache[location[0]]
            except KeyError:
                fspath = self.session.fspath.bestrelpath(location[0])
                cache[location[0]] = fspath
            location = (fspath, location[1], str(location[2]))
            self._location = location
            return location


class NoMatch(Exception):
    """ raised if matching cannot locate a matching names. """


class Interrupted(KeyboardInterrupt):
    """ signals an interrupted test run. """
    __module__ = 'builtins'  # for py3


class Failed(Exception):
    """ signals an stop as failed test run. """


class Session(FSCollector):
    Interrupted = Interrupted
    Failed = Failed

    def __init__(self, config):
        FSCollector.__init__(self, config.rootdir, parent=None,
                             config=config, session=self)
        self.testsfailed = 0
        self.testscollected = 0
        self.shouldstop = False
        self.shouldfail = False
        self.trace = config.trace.root.get("collection")
        self._norecursepatterns = config.getini("norecursedirs")
        self.startdir = py.path.local()
        self.config.pluginmanager.register(self, name="session")

    def _makeid(self):
        return ""

    @hookimpl(tryfirst=True)
    def pytest_collectstart(self):
        if self.shouldfail:
            raise self.Failed(self.shouldfail)
        if self.shouldstop:
            raise self.Interrupted(self.shouldstop)

    @hookimpl(tryfirst=True)
    def pytest_runtest_logreport(self, report):
        if report.failed and not hasattr(report, 'wasxfail'):
            self.testsfailed += 1
            maxfail = self.config.getvalue("maxfail")
            if maxfail and self.testsfailed >= maxfail:
                self.shouldfail = "stopping after %d failures" % (
                    self.testsfailed)
    pytest_collectreport = pytest_runtest_logreport

    def isinitpath(self, path):
        return path in self._initialpaths

    def gethookproxy(self, fspath):
        # check if we have the common case of running
        # hooks with all conftest.py filesall conftest.py
        pm = self.config.pluginmanager
        my_conftestmodules = pm._getconftestmodules(fspath)
        remove_mods = pm._conftest_plugins.difference(my_conftestmodules)
        if remove_mods:
            # one or more conftests are not in use at this fspath
            proxy = FSHookProxy(fspath, pm, remove_mods)
        else:
            # all plugis are active for this fspath
            proxy = self.config.hook
        return proxy

    def perform_collect(self, args=None, genitems=True):
        hook = self.config.hook
        try:
            items = self._perform_collect(args, genitems)
            self.config.pluginmanager.check_pending()
            hook.pytest_collection_modifyitems(session=self,
                                               config=self.config, items=items)
        finally:
            hook.pytest_collection_finish(session=self)
        self.testscollected = len(items)
        return items

    def _perform_collect(self, args, genitems):
        if args is None:
            args = self.config.args
        self.trace("perform_collect", self, args)
        self.trace.root.indent += 1
        self._notfound = []
        self._initialpaths = set()
        self._initialparts = []
        self.items = items = []
        for arg in args:
            parts = self._parsearg(arg)
            self._initialparts.append(parts)
            self._initialpaths.add(parts[0])
        rep = collect_one_node(self)
        self.ihook.pytest_collectreport(report=rep)
        self.trace.root.indent -= 1
        if self._notfound:
            errors = []
            for arg, exc in self._notfound:
                line = "(no name %r in any of %r)" % (arg, exc.args[0])
                errors.append("not found: %s\n%s" % (arg, line))
                # XXX: test this
            raise UsageError(*errors)
        if not genitems:
            return rep.result
        else:
            if rep.passed:
                for node in rep.result:
                    self.items.extend(self.genitems(node))
            return items

    def collect(self):
        for parts in self._initialparts:
            arg = "::".join(map(str, parts))
            self.trace("processing argument", arg)
            self.trace.root.indent += 1
            try:
                for x in self._collect(arg):
                    yield x
            except NoMatch:
                # we are inside a make_report hook so
                # we cannot directly pass through the exception
                self._notfound.append((arg, sys.exc_info()[1]))

            self.trace.root.indent -= 1

    def _collect(self, arg):
        names = self._parsearg(arg)
        path = names.pop(0)
        if path.check(dir=1):
            assert not names, "invalid arg %r" % (arg,)
            for path in path.visit(fil=lambda x: x.check(file=1),
                                   rec=self._recurse, bf=True, sort=True):
                for x in self._collectfile(path):
                    yield x
        else:
            assert path.check(file=1)
            for x in self.matchnodes(self._collectfile(path), names):
                yield x

    def _collectfile(self, path):
        ihook = self.gethookproxy(path)
        if not self.isinitpath(path):
            if ihook.pytest_ignore_collect(path=path, config=self.config):
                return ()
        return ihook.pytest_collect_file(path=path, parent=self)

    def _recurse(self, path):
        ihook = self.gethookproxy(path.dirpath())
        if ihook.pytest_ignore_collect(path=path, config=self.config):
            return
        for pat in self._norecursepatterns:
            if path.check(fnmatch=pat):
                return False
        ihook = self.gethookproxy(path)
        ihook.pytest_collect_directory(path=path, parent=self)
        return True

    def _tryconvertpyarg(self, x):
        """Convert a dotted module name to path.

        """
        import pkgutil
        try:
            loader = pkgutil.find_loader(x)
        except ImportError:
            return x
        if loader is None:
            return x
        # This method is sometimes invoked when AssertionRewritingHook, which
        # does not define a get_filename method, is already in place:
        try:
            path = loader.get_filename(x)
        except AttributeError:
            # Retrieve path from AssertionRewritingHook:
            path = loader.modules[x][0].co_filename
        if loader.is_package(x):
            path = os.path.dirname(path)
        return path

    def _parsearg(self, arg):
        """ return (fspath, names) tuple after checking the file exists. """
        parts = str(arg).split("::")
        if self.config.option.pyargs:
            parts[0] = self._tryconvertpyarg(parts[0])
        relpath = parts[0].replace("/", os.sep)
        path = self.config.invocation_dir.join(relpath, abs=True)
        if not path.check():
            if self.config.option.pyargs:
                raise UsageError(
                    "file or package not found: " + arg +
                    " (missing __init__.py?)")
            else:
                raise UsageError("file not found: " + arg)
        parts[0] = path
        return parts

    def matchnodes(self, matching, names):
        self.trace("matchnodes", matching, names)
        self.trace.root.indent += 1
        nodes = self._matchnodes(matching, names)
        num = len(nodes)
        self.trace("matchnodes finished -> ", num, "nodes")
        self.trace.root.indent -= 1
        if num == 0:
            raise NoMatch(matching, names[:1])
        return nodes

    def _matchnodes(self, matching, names):
        if not matching or not names:
            return matching
        name = names[0]
        assert name
        nextnames = names[1:]
        resultnodes = []
        for node in matching:
            if isinstance(node, Item):
                if not names:
                    resultnodes.append(node)
                continue
            assert isinstance(node, Collector)
            rep = collect_one_node(node)
            if rep.passed:
                has_matched = False
                for x in rep.result:
                    # TODO: remove parametrized workaround once collection structure contains parametrization
                    if x.name == name or x.name.split("[")[0] == name:
                        resultnodes.extend(self.matchnodes([x], nextnames))
                        has_matched = True
                # XXX accept IDs that don't have "()" for class instances
                if not has_matched and len(rep.result) == 1 and x.name == "()":
                    nextnames.insert(0, name)
                    resultnodes.extend(self.matchnodes([x], nextnames))
            else:
                # report collection failures here to avoid failing to run some test
                # specified in the command line because the module could not be
                # imported (#134)
                node.ihook.pytest_collectreport(report=rep)
        return resultnodes

    def genitems(self, node):
        self.trace("genitems", node)
        if isinstance(node, Item):
            node.ihook.pytest_itemcollected(item=node)
            yield node
        else:
            assert isinstance(node, Collector)
            rep = collect_one_node(node)
            if rep.passed:
                for subnode in rep.result:
                    for x in self.genitems(subnode):
                        yield x
            node.ihook.pytest_collectreport(report=rep)
