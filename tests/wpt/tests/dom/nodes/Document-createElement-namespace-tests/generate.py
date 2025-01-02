#!/usr/bin/python

import os
import sys

THIS_NAME = u"generate.py"

# Note: these lists must be kept in sync with the lists in
# Document-createElement-namespace.html, and this script must be run whenever
# the lists are updated.  (We could keep the lists in a shared JSON file, but
# seems like too much effort.)
FILES = (
    (u"empty", u""),
    (u"minimal_html", u"<!doctype html><title></title>"),

    (u"xhtml", u'<html xmlns="http://www.w3.org/1999/xhtml"></html>'),
    (u"svg", u'<svg xmlns="http://www.w3.org/2000/svg"></svg>'),
    (u"mathml", u'<mathml xmlns="http://www.w3.org/1998/Math/MathML"></mathml>'),

    (u"bare_xhtml", u"<html></html>"),
    (u"bare_svg", u"<svg></svg>"),
    (u"bare_mathml", u"<math></math>"),

    (u"xhtml_ns_removed", u"""\
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><script>
    var newRoot = document.createElementNS(null, "html");
    document.removeChild(document.documentElement);
    document.appendChild(newRoot);
  </script></head>
</html>
"""),
    (u"xhtml_ns_changed", u"""\
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
    u"html",
    u"xhtml",
    u"xml",
    u"svg",
    # Was not able to get server MIME type working properly :(
    #"mml",
)

def __main__():
    if len(sys.argv) > 1:
        print(u"No arguments expected, aborting")
        return

    if not os.access(THIS_NAME, os.F_OK):
        print(u"Must be run from the directory of " + THIS_NAME + u", aborting")
        return

    for name in os.listdir(u"."):
        if name == THIS_NAME:
            continue
        os.remove(name)

    manifest = open(u"MANIFEST", u"w")

    for name, contents in FILES:
        for extension in EXTENSIONS:
            f = open(name + u"." + extension, u"w")
            f.write(contents)
            f.close()
            manifest.write(u"support " + name + u"." + extension + u"\n")

    manifest.close()

__main__()
