# SPDX-License-Identifier: MIT

from __future__ import annotations

import json
import shutil
import subprocess

from pathlib import Path

import pytest

import attrs


pytestmark = [
    pytest.mark.skipif(
        shutil.which("pyright") is None, reason="Requires pyright."
    ),
]


@attrs.frozen
class PyrightDiagnostic:
    severity: str
    message: str


def parse_pyright_output(test_file: Path) -> set[PyrightDiagnostic]:
    pyright = subprocess.run(  # noqa: PLW1510
        ["pyright", "--outputjson", str(test_file)], capture_output=True
    )

    pyright_result = json.loads(pyright.stdout)

    return {
        PyrightDiagnostic(d["severity"], d["message"])
        for d in pyright_result["generalDiagnostics"]
    }


def test_pyright_baseline():
    """
    The typing.dataclass_transform decorator allows pyright to determine
    attrs decorated class types.
    """

    test_file = Path(__file__).parent / "dataclass_transform_example.py"

    diagnostics = parse_pyright_output(test_file)

    # Expected diagnostics as per pyright 1.1.311
    expected_diagnostics = {
        PyrightDiagnostic(
            severity="information",
            message='Type of "Define.__init__" is'
            ' "(self: Define, a: str, b: int) -> None"',
        ),
        PyrightDiagnostic(
            severity="information",
            message='Type of "DefineConverter.__init__" is '
            '"(self: DefineConverter, with_converter: str | Buffer | '
            'SupportsInt | SupportsIndex | SupportsTrunc) -> None"',
        ),
        PyrightDiagnostic(
            severity="error",
            message='Cannot assign member "a" for type '
            '"Frozen"\n\xa0\xa0"Frozen" is frozen\n\xa0\xa0\xa0\xa0Member "__set__" is unknown',
        ),
        PyrightDiagnostic(
            severity="information",
            message='Type of "d.a" is "Literal[\'new\']"',
        ),
        PyrightDiagnostic(
            severity="error",
            message='Cannot assign member "a" for type '
            '"FrozenDefine"\n\xa0\xa0"FrozenDefine" is frozen\n\xa0\xa0\xa0\xa0'
            'Member "__set__" is unknown',
        ),
        PyrightDiagnostic(
            severity="information",
            message='Type of "d2.a" is "Literal[\'new\']"',
        ),
        PyrightDiagnostic(
            severity="information",
            message='Type of "af.__init__" is "(_a: int) -> None"',
        ),
    }

    assert expected_diagnostics == diagnostics


def test_pyright_attrsinstance_compat(tmp_path):
    """
    Test that `AttrsInstance` is compatible with Pyright.
    """
    test_pyright_attrsinstance_compat_path = (
        tmp_path / "test_pyright_attrsinstance_compat.py"
    )
    test_pyright_attrsinstance_compat_path.write_text(
        """\
import attrs

# We can assign any old object to `AttrsInstance`.
foo: attrs.AttrsInstance = object()

reveal_type(attrs.AttrsInstance)
"""
    )

    diagnostics = parse_pyright_output(test_pyright_attrsinstance_compat_path)
    expected_diagnostics = {
        PyrightDiagnostic(
            severity="information",
            message='Type of "attrs.AttrsInstance" is "type[AttrsInstance]"',
        ),
    }
    assert diagnostics == expected_diagnostics
