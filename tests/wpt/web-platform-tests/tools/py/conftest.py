import py
import sys

pytest_plugins = 'doctest pytester'.split()

collect_ignore = ['build', 'doc/_build']


import os, py
pid = os.getpid()

def pytest_addoption(parser):
    group = parser.getgroup("pylib", "py lib testing options")
    group.addoption('--runslowtests',
           action="store_true", dest="runslowtests", default=False,
           help=("run slow tests"))

def pytest_funcarg__sshhost(request):
    val = request.config.getvalue("sshhost")
    if val:
        return val
    py.test.skip("need --sshhost option")
def pytest_generate_tests(metafunc):
    multi = getattr(metafunc.function, 'multi', None)
    if multi is not None:
        assert len(multi.kwargs) == 1
        for name, l in multi.kwargs.items():
            for val in l:
                metafunc.addcall(funcargs={name: val})
    elif 'anypython' in metafunc.funcargnames:
        for name in ('python2.4', 'python2.5', 'python2.6',
                     'python2.7', 'python3.1', 'pypy-c', 'jython'):
            metafunc.addcall(id=name, param=name)

# XXX copied from execnet's conftest.py - needs to be merged
winpymap = {
    'python2.7': r'C:\Python27\python.exe',
    'python2.6': r'C:\Python26\python.exe',
    'python2.5': r'C:\Python25\python.exe',
    'python2.4': r'C:\Python24\python.exe',
    'python3.1': r'C:\Python31\python.exe',
}

def getexecutable(name, cache={}):
    try:
        return cache[name]
    except KeyError:
        executable = py.path.local.sysfind(name)
        if executable:
            if name == "jython":
                import subprocess
                popen = subprocess.Popen([str(executable), "--version"],
                    universal_newlines=True, stderr=subprocess.PIPE)
                out, err = popen.communicate()
                if not err or "2.5" not in err:
                    executable = None
        cache[name] = executable
        return executable

def pytest_funcarg__anypython(request):
    name = request.param
    executable = getexecutable(name)
    if executable is None:
        if sys.platform == "win32":
            executable = winpymap.get(name, None)
            if executable:
                executable = py.path.local(executable)
                if executable.check():
                    return executable
        py.test.skip("no %s found" % (name,))
    return executable
