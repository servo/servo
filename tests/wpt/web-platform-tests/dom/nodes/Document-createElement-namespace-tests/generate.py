#!/usr/bin/python

from __future__ import print_function

import os
import sys

THIS_NAME = "generate.py"

# Note: these lists must be kept in sync with the lists in
# Document-createElement-namespace.html, and this script must be run whenever
# the lists are updated.  (We could keep the lists in a shared JSON file, but
# seems like too much effort.)
FILES = (
    ("empty", ""),
    ("minimal_html", "<!doctype html><title></title>"),

    ("xhtml", '<html xmlns="http://www.w3.org/1999/xhtml"></html>'),
    ("svg", '<svg xmlns="http://www.w3.org/2000/svg"></svg>'),
    ("mathml", '<mathml xmlns="http://www.w3.org/1998/Math/MathML"></mathml>'),

    ("bare_xhtml", "<html></html>"),
    ("bare_svg", "<svg></svg>"),
    ("bare_mathml", "<math></math>"),

    ("xhtml_ns_removed", """\
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><script>
    var newRoot = document.createElementNS(null, "html");
    document.removeChild(document.documentElement);
    document.appendChild(newRoot);
  </script></head>
</html>
"""),
    ("xhtml_ns_changed", """\
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><script>
    var newRoot = document.createElementNS("http://www.w3.org/2000/svg", "abc");
    document.removeChild(document.documentElement);
    document.appendChild(newRoot);
  </script></head>
</html>
"""),
)

EXTENSIONS = (
    "html",
    "xhtml",
    "xml",
    "svg",
    # Was not able to get server MIME type working properly :(
    #"mml",
)

def __main__():
    if len(sys.argv) > 1:
        print("No arguments expected, aborting")
        return

    if not os.access(THIS_NAME, os.F_OK):
        print("Must be run from the directory of " + THIS_NAME + ", aborting")
        return

    for name in os.listdir("."):
        if name == THIS_NAME:
            continue
        os.remove(name)

    manifest = open("MANIFEST", "w")

    for name, contents in FILES:
        for extension in EXTENSIONS:
            f = open(name + "." + extension, "w")
            f.write(contents)
            f.close()
            manifest.write("support " + name + "." + extension + "\n")

    manifest.close()

__main__()
