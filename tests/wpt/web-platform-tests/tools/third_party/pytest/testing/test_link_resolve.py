import os.path
import subprocess
import sys
import textwrap
from contextlib import contextmanager
from pathlib import Path
from string import ascii_lowercase

from _pytest.pytester import Pytester


@contextmanager
def subst_path_windows(filepath: Path):
    for c in ascii_lowercase[7:]:  # Create a subst drive from H-Z.
        c += ":"
        if not os.path.exists(c):
            drive = c
            break
    else:
        raise AssertionError("Unable to find suitable drive letter for subst.")

    directory = filepath.parent
    basename = filepath.name

    args = ["subst", drive, str(directory)]
    subprocess.check_call(args)
    assert os.path.exists(drive)
    try:
        filename = Path(drive, os.sep, basename)
        yield filename
    finally:
        args = ["subst", "/D", drive]
        subprocess.check_call(args)


@contextmanager
def subst_path_linux(filepath: Path):
    directory = filepath.parent
    basename = filepath.name

    target = directory / ".." / "sub2"
    os.symlink(str(directory), str(target), target_is_directory=True)
    try:
        filename = target / basename
        yield filename
    finally:
        # We don't need to unlink (it's all in the tempdir).
        pass


def test_link_resolve(pytester: Pytester) -> None:
    """See: https://github.com/pytest-dev/pytest/issues/5965."""
    sub1 = pytester.mkpydir("sub1")
    p = sub1.joinpath("test_foo.py")
    p.write_text(
        textwrap.dedent(
            """
        import pytest
        def test_foo():
            raise AssertionError()
        """
        )
    )

    subst = subst_path_linux
    if sys.platform == "win32":
        subst = subst_path_windows

    with subst(p) as subst_p:
        result = pytester.runpytest(str(subst_p), "-v")
        # i.e.: Make sure that the error is reported as a relative path, not as a
        # resolved path.
        # See: https://github.com/pytest-dev/pytest/issues/5965
        stdout = result.stdout.str()
        assert "sub1/test_foo.py" not in stdout

        # i.e.: Expect drive on windows because we just have drive:filename, whereas
        # we expect a relative path on Linux.
        expect = f"*{subst_p}*" if sys.platform == "win32" else "*sub2/test_foo.py*"
        result.stdout.fnmatch_lines([expect])
