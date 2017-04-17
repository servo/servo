#!/usr/bin/python

# HTMLMake
# Converts all files of specified extension from XHTML to HTML
# Written by fantasai
# Joint copyright 2010 W3C and Microsoft
# Licensed under BSD 3-Clause: <http://www.w3.org/Consortium/Legal/2008/03-bsd-license>

srcExt = '.xht'
dstExt = '.htm'
skipDirs = ('contributors/microsoft/submitted/support', # XXXfixme files should be .xht
            'incoming', '.svn', 'CVS', '.hg')

import os.path
from os.path import join, getmtime
import sys
import re
import os
from w3ctestlib.Sources import XHTMLSource, SourceTree

def xhtml2html(source, dest):
    """Convert XHTML file given by path `source` into HTML file at path `dest`."""

    # Parse and serialize
    xs = XHTMLSource(SourceTree(None), source, dest)
    o = xs.serializeHTML()

    # Report errors
    if xs.error:
      print >>sys.stderr, "Error parsing XHTML file %s: %s" % (source, xs.error)

    # Write
    f = open(dest, 'w')
    f.write(o.encode(xs.encoding, 'xmlcharrefreplace'))
    f.close()

if len(sys.argv) == 3:
    clobber = sys.argv[1] == '--clobber'
    force   = sys.argv[1] == '-f'
    root    = sys.argv[2]
elif len(sys.argv) == 2 and (sys.argv[1] != '--clobber' and sys.argv[1] != '-f'):
    clobber = False;
    force   = False;
    root    = sys.argv[1]
else:
    print "make-html converts all %s XHTML files to %s HTML files." % (srcExt, dstExt)
    print "Only changed files are converted, unless you specify -f."
    print "To use, specify the root directory of the files you want converted, e.g."
    print "  make-html ."
    print "To delete all files with extension %s, specify the --clobber option." % dstExt
    exit()

for root, dirs, files in os.walk(root):
    for skip in skipDirs:
        if skip in dirs:
            dirs.remove(skip)
    for file in files:
        if clobber:
            if file.endswith(dstExt):
                os.remove(join(root, file))
        elif file.endswith(srcExt):
            source = join(root, file)
            dest = join(root, file[0:-1*len(srcExt)] + dstExt)
            if not os.path.exists(dest) or getmtime(source) > getmtime(dest) or force:
                # print "Processing %s" % source
                xhtml2html(source, dest)
