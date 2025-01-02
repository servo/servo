import os
import subprocess
import sys
from typing import Any, Callable, Generic, Optional, Text, TypeVar
T = TypeVar("T")


def rel_path_to_url(rel_path: Text, url_base: Text = "/") -> Text:
    assert not os.path.isabs(rel_path), rel_path
    if url_base[0] != "/":
        url_base = "/" + url_base
    if url_base[-1] != "/":
        url_base += "/"
    return url_base + rel_path.replace(os.sep, "/")


def from_os_path(path: Text) -> Text:
    assert os.path.sep == "/" or sys.platform == "win32"
    if "/" == os.path.sep:
        rv = path
    else:
        rv = path.replace(os.path.sep, "/")
    if "\\" in rv:
        raise ValueError("path contains \\ when separator is %s" % os.path.sep)
    return rv


def to_os_path(path: Text) -> Text:
    assert os.path.sep == "/" or sys.platform == "win32"
    if "\\" in path:
        raise ValueError("normalised path contains \\")
    if "/" == os.path.sep:
        return path
    return path.replace("/", os.path.sep)


def git(path: Text) -> Optional[Callable[..., Text]]:
    def gitfunc(cmd: Text, *args: Text) -> Text:
        full_cmd = ["git", cmd] + list(args)
        try:
            return subprocess.check_output(full_cmd, cwd=path, stderr=subprocess.STDOUT).decode('utf8')
        except Exception as e:
            if sys.platform == "win32" and isinstance(e, WindowsError):
                full_cmd[0] = "git.bat"
                return subprocess.check_output(full_cmd, cwd=path, stderr=subprocess.STDOUT).decode('utf8')
            else:
                raise

    try:
        # this needs to be a command that fails if we aren't in a git repo
        gitfunc("rev-parse", "--show-toplevel")
    except (subprocess.CalledProcessError, OSError):
        return None
    else:
        return gitfunc


class cached_property(Generic[T]):
    def __init__(self, func: Callable[[Any], T]) -> None:
        self.func = func
        self.__doc__ = getattr(func, "__doc__")
        self.name = func.__name__

    def __get__(self, obj: Any, cls: Optional[type] = None) -> T:
        if obj is None:
            return self  # type: ignore

        # we can unconditionally assign as next time this won't be called
        assert self.name not in obj.__dict__
        rv = obj.__dict__[self.name] = self.func(obj)
        obj.__dict__.setdefault("__cached_properties__", set()).add(self.name)
        return rv
