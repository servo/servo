import os
import platform
import subprocess

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Text
    from typing import Callable
    from typing import Any
    from typing import Generic
    from typing import TypeVar
    from typing import Optional
    T = TypeVar("T")
else:
    # eww, eww, ewwww
    Generic = {}
    T = object()
    Generic[T] = object


def rel_path_to_url(rel_path, url_base="/"):
    # type: (Text, Text) -> Text
    assert not os.path.isabs(rel_path), rel_path
    if url_base[0] != u"/":
        url_base = u"/" + url_base
    if url_base[-1] != u"/":
        url_base += u"/"
    return url_base + rel_path.replace(os.sep, u"/")


def from_os_path(path):
    # type: (Text) -> Text
    assert os.path.sep == u"/" or platform.system() == "Windows"
    if u"/" == os.path.sep:
        rv = path
    else:
        rv = path.replace(os.path.sep, u"/")
    if u"\\" in rv:
        raise ValueError("path contains \\ when separator is %s" % os.path.sep)
    return rv


def to_os_path(path):
    # type: (Text) -> Text
    assert os.path.sep == u"/" or platform.system() == "Windows"
    if u"\\" in path:
        raise ValueError("normalised path contains \\")
    if u"/" == os.path.sep:
        return path
    return path.replace(u"/", os.path.sep)


def git(path):
    # type: (Text) -> Optional[Callable[..., Text]]
    def gitfunc(cmd, *args):
        # type: (Text, *Text) -> Text
        full_cmd = [u"git", cmd] + list(args)
        try:
            return subprocess.check_output(full_cmd, cwd=path, stderr=subprocess.STDOUT).decode('utf8')
        except Exception as e:
            if platform.uname()[0] == "Windows" and isinstance(e, WindowsError):
                full_cmd[0] = u"git.bat"
                return subprocess.check_output(full_cmd, cwd=path, stderr=subprocess.STDOUT).decode('utf8')
            else:
                raise

    try:
        # this needs to be a command that fails if we aren't in a git repo
        gitfunc(u"rev-parse", u"--show-toplevel")
    except (subprocess.CalledProcessError, OSError):
        return None
    else:
        return gitfunc


class cached_property(Generic[T]):
    def __init__(self, func):
        # type: (Callable[[Any], T]) -> None
        self.func = func
        self.__doc__ = getattr(func, "__doc__")
        self.name = func.__name__

    def __get__(self, obj, cls=None):
        # type: (Any, Optional[type]) -> T
        if obj is None:
            return self  # type: ignore

        # we can unconditionally assign as next time this won't be called
        assert self.name not in obj.__dict__
        rv = obj.__dict__[self.name] = self.func(obj)
        obj.__dict__.setdefault("__cached_properties__", set()).add(self.name)
        return rv
