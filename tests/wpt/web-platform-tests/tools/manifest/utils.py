import os
import platform
import subprocess

from six import BytesIO

def rel_path_to_url(rel_path, url_base="/"):
    assert not os.path.isabs(rel_path), rel_path
    if url_base[0] != "/":
        url_base = "/" + url_base
    if url_base[-1] != "/":
        url_base += "/"
    return url_base + rel_path.replace(os.sep, "/")


def from_os_path(path):
    assert os.path.sep == "/" or platform.system() == "Windows"
    if "/" == os.path.sep:
        rv = path
    else:
        rv = path.replace(os.path.sep, "/")
    if "\\" in rv:
        raise ValueError("path contains \\ when separator is %s" % os.path.sep)
    return rv


def to_os_path(path):
    assert os.path.sep == "/" or platform.system() == "Windows"
    if "\\" in path:
        raise ValueError("normalised path contains \\")
    if "/" == os.path.sep:
        return path
    return path.replace("/", os.path.sep)


def git(path):
    def gitfunc(cmd, *args):
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


class ContextManagerBytesIO(BytesIO):
    def __enter__(self):
        return self

    def __exit__(self, *args, **kwargs):
        self.close()


class cached_property(object):
    def __init__(self, func):
        self.func = func
        self.__doc__ = getattr(func, "__doc__")
        self.name = func.__name__

    def __get__(self, obj, cls=None):
        if obj is None:
            return self

        if self.name not in obj.__dict__:
            obj.__dict__[self.name] = self.func(obj)
            obj.__dict__.setdefault("__cached_properties__", set()).add(self.name)
        return obj.__dict__[self.name]
