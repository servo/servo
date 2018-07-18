import py
import subprocess
import sys
import pytest
import _pytest

MODSET = [
    x
    for x in py.path.local(_pytest.__file__).dirpath().visit("*.py")
    if x.purebasename != "__init__"
]


@pytest.mark.parametrize("modfile", MODSET, ids=lambda x: x.purebasename)
def test_fileimport(modfile):
    # this test ensures all internal packages can import
    # without needing the pytest namespace being set
    # this is critical for the initialization of xdist

    res = subprocess.call(
        [
            sys.executable,
            "-c",
            "import sys, py; py.path.local(sys.argv[1]).pyimport()",
            modfile.strpath,
        ]
    )
    if res:
        pytest.fail("command result %s" % res)
