#!/usr/local/bin/python
# yply.py
#
# Author: David Beazley (dave@dabeaz.com)
# Date  : October 2, 2006
#
# Converts a UNIX-yacc specification file into a PLY-compatible
# specification.   To use, simply do this:
#
#   % python yply.py [-nocode] inputfile.y >myparser.py
#
# The output of this program is Python code. In the output,
# any C code in the original file is included, but is commented.
# If you use the -nocode option, then all of the C code in the
# original file is discarded.
#
# Disclaimer:  This just an example I threw together in an afternoon.
# It might have some bugs.  However, it worked when I tried it on
# a yacc-specified C++ parser containing 442 rules and 855 parsing
# states.
#

import sys
sys.path.insert(0, "../..")

import ylex
import yparse

from ply import *

if len(sys.argv) == 1:
    print("usage : yply.py [-nocode] inputfile")
    raise SystemExit

if len(sys.argv) == 3:
    if sys.argv[1] == '-nocode':
        yparse.emit_code = 0
    else:
        print("Unknown option '%s'" % sys.argv[1])
        raise SystemExit
    filename = sys.argv[2]
else:
    filename = sys.argv[1]

yacc.parse(open(filename).read())

print("""
if __name__ == '__main__':
    from ply import *
    yacc.yacc()
""")
