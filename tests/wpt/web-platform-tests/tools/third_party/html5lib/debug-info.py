from __future__ import print_function, unicode_literals

import platform
import sys


info = {
    "impl": platform.python_implementation(),
    "version": platform.python_version(),
    "revision": platform.python_revision(),
    "maxunicode": sys.maxunicode,
    "maxsize": sys.maxsize
}

search_modules = ["chardet", "datrie", "genshi", "html5lib", "lxml", "six"]
found_modules = []

for m in search_modules:
    try:
        __import__(m)
    except ImportError:
        pass
    else:
        found_modules.append(m)

info["modules"] = ", ".join(found_modules)


print("""html5lib debug info:

Python %(version)s (revision: %(revision)s)
Implementation: %(impl)s

sys.maxunicode: %(maxunicode)X
sys.maxsize: %(maxsize)X

Installed modules: %(modules)s""" % info)
