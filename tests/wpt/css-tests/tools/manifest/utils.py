import os
import urlparse
from StringIO import StringIO

blacklist = ["/", "/tools/", "/resources/", "/common/", "/conformance-checkers/", "_certs"]

def rel_path_to_url(rel_path, url_base="/"):
    assert not os.path.isabs(rel_path)
    if url_base[0] != "/":
        url_base = "/" + url_base
    if url_base[-1] != "/":
        url_base += "/"
    return url_base + rel_path.replace(os.sep, "/")

def is_blacklisted(url):
    for item in blacklist:
        if item == "/":
            if "/" not in url[1:]:
                return True
        elif url.startswith(item):
            return True
    return False

class ContextManagerStringIO(StringIO):
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
