# SPDX-License-Identifier: MIT

import json
import os.path
import shutil
import subprocess
import sys

import pytest

import attr


if sys.version_info < (3, 6):
    _found_pyright = False
else:
    _found_pyright = shutil.which("pyright")


@attr.s(frozen=True)
class PyrightDiagnostic(object):
    severity = attr.ib()
    message = attr.ib()


@pytest.mark.skipif(not _found_pyright, reason="Requires pyright.")
def test_pyright_baseline():
    """The __dataclass_transform__ decorator allows pyright to determine
    attrs decorated class types.
    """

    test_file = os.path.dirname(__file__) + "/dataclass_transform_example.py"

    pyright = subprocess.run(
        ["pyright", "--outputjson", str(test_file)], capture_output=True
    )
    pyright_result = json.loads(pyright.stdout)

    diagnostics = set(
        PyrightDiagnostic(d["severity"], d["message"])
        for d in pyright_result["generalDiagnostics"]
    )

    # Expected diagnostics as per pyright 1.1.135
    expected_diagnostics = {
        PyrightDiagnostic(
            severity="information",
            message='Type of "Define.__init__" is'
            ' "(self: Define, a: str, b: int) -> None"',
        ),
        PyrightDiagnostic(
            severity="information",
            message='Type of "DefineConverter.__init__" is '
            '"(self: DefineConverter, with_converter: int) -> None"',
        ),
        PyrightDiagnostic(
            severity="information",
            message='Type of "d.a" is "Literal[\'new\']"',
        ),
        PyrightDiagnostic(
            severity="error",
            message='Cannot assign member "a" for type '
            '"FrozenDefine"\n\xa0\xa0"FrozenDefine" is frozen',
        ),
        PyrightDiagnostic(
            severity="information",
            message='Type of "d2.a" is "Literal[\'new\']"',
        ),
    }

    assert diagnostics == expected_diagnostics
