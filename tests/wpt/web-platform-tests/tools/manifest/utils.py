import os
import platform
import subprocess

from six import BytesIO

MYPY = False
if MYPY:
    # MYPY is set to True when run under Mypy.
    from typing import Text
    from typing import Callable
    from typing import AnyStr
    from typing import Any
    from typing import BinaryIO
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
    # type: (bytes, Text) -> Text
    assert not os.path.isabs(rel_path), rel_path
    if url_base[0] != "/":
        url_base = "/" + url_base
    if url_base[-1] != "/":
        url_base += "/"
    return url_base + rel_path.replace(os.sep, "/")


def from_os_path(path):
    # type: (AnyStr) -> AnyStr
    assert os.path.sep == "/" or platform.system() == "Windows"
    if "/" == os.path.sep:
        rv = path
    else:
        rv = path.replace(os.path.sep, "/")
    if "\\" in rv:
        raise ValueError("path contains \\ when separator is %s" % os.path.sep)
    return rv


def to_os_path(path):
    # type: (AnyStr) -> AnyStr
    assert os.path.sep == "/" or platform.system() == "Windows"
    if "\\" in path:
        raise ValueError("normalised path contains \\")
    if "/" == os.path.sep:
        return path
    return path.replace("/", os.path.sep)


def git(path):
    # type: (bytes) -> Optional[Callable[..., bytes]]
    def gitfunc(cmd, *args):
        # type: (bytes, *bytes) -> bytes
        full_cmd = ["git", cmd] + list(args)
        try:
            return subprocess.check_output(full_cmd, cwd=path, stderr=subprocess.STDOUT)
        except Exception as e:
            if platform.uname()[0] == "Windows" and isinstance(e, WindowsError):
                full_cmd[0] = "git.bat"
                return subprocess.check_output(full_cmd, cwd=path, stderr=subprocess.STDOUT)
            else:
                raise

    try:
        # this needs to be a command that fails if we aren't in a git repo
        gitfunc("rev-parse", "--show-toplevel")
    except (subprocess.CalledProcessError, OSError):
        return None
    else:
        return gitfunc


class ContextManagerBytesIO(BytesIO):  # type: ignore
    def __enter__(self):
        # type: () -> BinaryIO
        return self  # type: ignore

    def __exit__(self, *args, **kwargs):
        # type: (*Any, **Any) -> bool
        self.close()
        return True


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
        return rv
