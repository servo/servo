import platform
import os

from six import BytesIO

def rel_path_to_url(rel_path, url_base="/"):
    assert not os.path.isabs(rel_path)
    if url_base[0] != "/":
        url_base = "/" + url_base
    if url_base[-1] != "/":
        url_base += "/"
    return url_base + rel_path.replace(os.sep, "/")


def from_os_path(path):
    assert os.path.sep == "/" or platform.system() == "Windows"
    rv = path.replace(os.path.sep, "/")
    if "\\" in rv:
        raise ValueError("path contains \\ when separator is %s" % os.path.sep)
    return rv


def to_os_path(path):
    assert os.path.sep == "/" or platform.system() == "Windows"
    if "\\" in path:
        raise ValueError("normalised path contains \\")
    return path.replace("/", os.path.sep)


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
