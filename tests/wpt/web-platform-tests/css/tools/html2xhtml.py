#!/usr/bin/python

# This file is licensed under CC Zero

import sys
import html5lib
import re

if len(sys.argv) != 3:
  print """! html2xhtml requires two arguments: the filename to read, and the filename to write"""
  exit()

#######################################################################
# Parse HTML and output XHTML

f = open(sys.argv[1])
p = html5lib.HTMLParser()
t = p.parse(f)
o = html5lib.serializer.serialize(t, format='xhtml')
f.close()

#######################################################################
# Clean up the mess left by html5lib

def firstMatch(m): # Python makes s/x(y+)?/z$1/ very difficult
  if m.group(1):
    return m.group(1)
  return ''

# Missing XHTML artifacts

o = re.sub('<!DOCTYPE [^>]+>',
           '<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd">',
           o);
o = re.sub('<html( [^>]+)?>',
           lambda m : '<html' + firstMatch(m) + ' xmlns="http://www.w3.org/1999/xhtml">',
           o);

# Fix weird reordering

o = re.sub('<link href="(.*?)" (.*?) ?/>',
           lambda m : '<link ' + m.group(2) + ' href="' + m.group(1) + '"/>',
           o);

# Indentation

o = re.sub('<!DOCTYPE ([^>]+)><html',
           lambda m : '<!DOCTYPE ' +  firstMatch(m) + '>\n<html',
           o);
o = re.sub('<html( [^>]+)?><',
           lambda m : '<html' + firstMatch(m) + '>\n<',
           o);
o = re.sub('<head( [^>]+)?><',
           lambda m : '<head' + firstMatch(m) + '>\n<',
           o);
o = re.sub('</head><',
           '</head>\n<',
           o);
o = re.sub('<body( [^>]+)?><',
           lambda m : '<body' + firstMatch(m) + '>\n<',
           o);
o = re.sub('</body><',
           '</body>\n<',
           o);
o = re.sub('</html>$',
           '</html>\n',
           o);
o = re.sub('\xa0',
           '&nbsp;',
           o); # make nbsp visible to people viewing source

#######################################################################
# Write to file

f = open(sys.argv[2], 'w')
f.write(o.encode('utf-8'))
f.close()
