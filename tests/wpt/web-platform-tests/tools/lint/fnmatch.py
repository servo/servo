from __future__ import absolute_import

import fnmatch as _stdlib_fnmatch
import os


__all__ = ["fnmatch", "fnmatchcase", "filter", "translate"]


def fnmatch(name, pat):
    name = os.path.normcase(name)
    pat = os.path.normcase(pat)
    return fnmatchcase(name, pat)


def fnmatchcase(name, pat):
    if '?' not in pat and '[' not in pat:
        wildcards = pat.count("*")
        if wildcards == 0:
            return name == pat
        elif wildcards == 1 and pat[0] == "*":
            return name.endswith(pat[1:])
        elif wildcards == 1 and pat[-1] == "*":
            return name.startswith(pat[:-1])
    return _stdlib_fnmatch.fnmatchcase(name, pat)


def filter(names, pat):
    return [n for n in names if fnmatch(n, pat)]


translate = _stdlib_fnmatch.translate
