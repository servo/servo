"""Module containing a parametrized tests testing cross-python serialization
via the pickle module."""

import shutil
import subprocess
import textwrap

import pytest


pythonlist = ["python3.9", "python3.10", "python3.11"]


@pytest.fixture(params=pythonlist)
def python1(request, tmp_path):
    picklefile = tmp_path / "data.pickle"
    return Python(request.param, picklefile)


@pytest.fixture(params=pythonlist)
def python2(request, python1):
    return Python(request.param, python1.picklefile)


class Python:
    def __init__(self, version, picklefile):
        self.pythonpath = shutil.which(version)
        if not self.pythonpath:
            pytest.skip(f"{version!r} not found")
        self.picklefile = picklefile

    def dumps(self, obj):
        dumpfile = self.picklefile.with_name("dump.py")
        dumpfile.write_text(
            textwrap.dedent(
                rf"""
                import pickle
                f = open({str(self.picklefile)!r}, 'wb')
                s = pickle.dump({obj!r}, f, protocol=2)
                f.close()
                """
            )
        )
        subprocess.run((self.pythonpath, str(dumpfile)), check=True)

    def load_and_is_true(self, expression):
        loadfile = self.picklefile.with_name("load.py")
        loadfile.write_text(
            textwrap.dedent(
                rf"""
                import pickle
                f = open({str(self.picklefile)!r}, 'rb')
                obj = pickle.load(f)
                f.close()
                res = eval({expression!r})
                if not res:
                    raise SystemExit(1)
                """
            )
        )
        print(loadfile)
        subprocess.run((self.pythonpath, str(loadfile)), check=True)


@pytest.mark.parametrize("obj", [42, {}, {1: 3}])
def test_basic_objects(python1, python2, obj):
    python1.dumps(obj)
    python2.load_and_is_true(f"obj == {obj}")
