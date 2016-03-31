import pytest
import sys


@pytest.fixture(scope="module")
def standalone(request):
    return Standalone(request)

class Standalone:
    def __init__(self, request):
        self.testdir = request.getfuncargvalue("testdir")
        script = "mypytest"
        result = self.testdir.runpytest("--genscript=%s" % script)
        assert result.ret == 0
        self.script = self.testdir.tmpdir.join(script)
        assert self.script.check()

    def run(self, anypython, testdir, *args):
        return testdir._run(anypython, self.script, *args)

def test_gen(testdir, anypython, standalone):
    if sys.version_info >= (2,7):
        result = testdir._run(anypython, "-c",
                                "import sys;print (sys.version_info >=(2,7))")
        assert result.ret == 0
        if result.stdout.str() == "False":
            pytest.skip("genscript called from python2.7 cannot work "
                        "earlier python versions")
    result = standalone.run(anypython, testdir, '--version')
    if result.ret == 2:
        result.stderr.fnmatch_lines(["*ERROR: setuptools not installed*"])
    elif result.ret == 0:
        result.stderr.fnmatch_lines([
            "*imported from*mypytest*"
        ])
        p = testdir.makepyfile("def test_func(): assert 0")
        result = standalone.run(anypython, testdir, p)
        assert result.ret != 0
    else:
        pytest.fail("Unexpected return code")


def test_freeze_includes():
    """
    Smoke test for freeze_includes(), to ensure that it works across all
    supported python versions.
    """
    includes = pytest.freeze_includes()
    assert len(includes) > 1
    assert '_pytest.genscript' in includes

