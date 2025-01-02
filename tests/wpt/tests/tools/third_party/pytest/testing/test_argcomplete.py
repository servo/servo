# mypy: allow-untyped-defs
from pathlib import Path
import subprocess
import sys

from _pytest.monkeypatch import MonkeyPatch
import pytest


# Test for _argcomplete but not specific for any application.


def equal_with_bash(prefix, ffc, fc, out=None):
    res = ffc(prefix)
    res_bash = set(fc(prefix))
    retval = set(res) == res_bash
    if out:
        out.write(f"equal_with_bash({prefix}) {retval} {res}\n")
        if not retval:
            out.write(" python - bash: %s\n" % (set(res) - res_bash))
            out.write(" bash - python: %s\n" % (res_bash - set(res)))
    return retval


# Copied from argcomplete.completers as import from there.
# Also pulls in argcomplete.__init__ which opens filedescriptor 9.
# This gives an OSError at the end of testrun.


def _wrapcall(*args, **kargs):
    try:
        return subprocess.check_output(*args, **kargs).decode().splitlines()
    except subprocess.CalledProcessError:
        return []


class FilesCompleter:
    """File completer class, optionally takes a list of allowed extensions."""

    def __init__(self, allowednames=(), directories=True):
        # Fix if someone passes in a string instead of a list
        if type(allowednames) is str:
            allowednames = [allowednames]

        self.allowednames = [x.lstrip("*").lstrip(".") for x in allowednames]
        self.directories = directories

    def __call__(self, prefix, **kwargs):
        completion = []
        if self.allowednames:
            if self.directories:
                files = _wrapcall(["bash", "-c", f"compgen -A directory -- '{prefix}'"])
                completion += [f + "/" for f in files]
            for x in self.allowednames:
                completion += _wrapcall(
                    ["bash", "-c", f"compgen -A file -X '!*.{x}' -- '{prefix}'"]
                )
        else:
            completion += _wrapcall(["bash", "-c", f"compgen -A file -- '{prefix}'"])

            anticomp = _wrapcall(["bash", "-c", f"compgen -A directory -- '{prefix}'"])

            completion = list(set(completion) - set(anticomp))

            if self.directories:
                completion += [f + "/" for f in anticomp]
        return completion


class TestArgComplete:
    @pytest.mark.skipif("sys.platform in ('win32', 'darwin')")
    def test_compare_with_compgen(
        self, tmp_path: Path, monkeypatch: MonkeyPatch
    ) -> None:
        from _pytest._argcomplete import FastFilesCompleter

        ffc = FastFilesCompleter()
        fc = FilesCompleter()

        monkeypatch.chdir(tmp_path)

        assert equal_with_bash("", ffc, fc, out=sys.stdout)

        tmp_path.cwd().joinpath("data").touch()

        for x in ["d", "data", "doesnotexist", ""]:
            assert equal_with_bash(x, ffc, fc, out=sys.stdout)

    @pytest.mark.skipif("sys.platform in ('win32', 'darwin')")
    def test_remove_dir_prefix(self):
        """This is not compatible with compgen but it is with bash itself: ls /usr/<TAB>."""
        from _pytest._argcomplete import FastFilesCompleter

        ffc = FastFilesCompleter()
        fc = FilesCompleter()
        for x in "/usr/".split():
            assert not equal_with_bash(x, ffc, fc, out=sys.stdout)
