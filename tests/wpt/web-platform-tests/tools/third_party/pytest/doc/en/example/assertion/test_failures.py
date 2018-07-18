
import py

failure_demo = py.path.local(__file__).dirpath("failure_demo.py")
pytest_plugins = "pytester",


def test_failure_demo_fails_properly(testdir):
    target = testdir.tmpdir.join(failure_demo.basename)
    failure_demo.copy(target)
    failure_demo.copy(testdir.tmpdir.join(failure_demo.basename))
    result = testdir.runpytest(target, syspathinsert=True)
    result.stdout.fnmatch_lines(["*42 failed*"])
    assert result.ret != 0
