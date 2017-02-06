#!/usr/bin/env python

import py

def get_version_string():
    fn = py.path.local(__file__).join("..", "..", "..",
                                      "_pytest", "__init__.py")
    for line in fn.readlines():
        if "version" in line and not line.strip().startswith('#'):
            return eval(line.split("=")[-1])

def get_minor_version_string():
    return ".".join(get_version_string().split(".")[:2])

if __name__ == "__main__":
    print (get_minor_version_string())
