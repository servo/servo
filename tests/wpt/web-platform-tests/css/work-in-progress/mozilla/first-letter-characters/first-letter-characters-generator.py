#!/usr/bin/python

# This script authored by L. David Baron (Mozilla Corporation)

import re
from math import floor
from urllib import urlopen

# unicode data tools
unicodedb = open("UnicodeData.txt")
# unicodedb = urlopen("http://www.unicode.org/Public/UNIDATA/UnicodeData.txt")

isSurrogateOrPUA = re.compile('^<.*(Private Use|Surrogate).*>$').search
isPunctuation = re.compile('^P(s|e|i|f|o)$').search

def isValidXMLChar(charcode):
    return (charcode == 0x9) or \
           (charcode  == 0xA) or \
           (charcode == 0xD) or \
           (charcode >= 0x20 and charcode <= 0xD7FF) or \
           (charcode >= 0xE000 and charcode <= 0xFFFD) or \
           (charcode >= 0x10000 and charcode <= 0x10FFFF)

def escape(codepoint):
  return "&#x" + hex(codepoint)[2:] + ";"


class UnicodeTestGenerator:

    # config
    fileheader = open("first-letter-characters.tmpl").read()
    filefooter = "</body></html>"
    fileprefix = "first-letter-punct-before-"
    filesuffix = ".xht"
    blocksize = 1024
    linesize = 256

    def writeTest(self, charcode, ispunct):
        self.linecount += 1
        if self.linecount == self.linesize:
            self.linecount = 0
            self.out.write("<hr/>\n")
        if ispunct:
            classname = "  extend  "
        else:
            classname = "dontextend"
        self.out.write("<div class=\"test " + classname + "\"><div>" + \
            escape(charcode) + "C<span class=\"spacer\"></span></div></div\n>")

    def startNewFile(self):
        if self.out:
           self.out.write(self.filefooter)
           self.out.close()
        self.filenum += 1
        self.out = open(self.fileprefix + "%03d" % self.filenum + self.filesuffix, 'w')
        self.out.write(self.fileheader)

    def write(self):
        self.linecount = 0
        self.out = None
        self.filenum = 0

        rangefirst = None
        breakat = 0
        for line in unicodedb:
            fields = line.split(";")
            charcode = int(fields[0], 16)
            charname = fields[1]
            ispunct = isPunctuation(fields[2])
            if isSurrogateOrPUA(charname) or not isValidXMLChar(charcode):
                pass
            elif charname.endswith(", First>"):
                if rangefirst != None:
                    raise SyntaxError
                rangefirst = charcode
            elif charname.endswith(", Last>"):
                if rangefirst == None:
                    raise SyntaxError
                if rangefirst >= breakat:
                    # we've exceeded our chunking size; break to a new file
                    self.startNewFile()
                    print "Break at %x" % breakat
                    breakat = (floor(rangefirst / self.blocksize) + 1) * self.blocksize
                    print "Next break at %x" % breakat
                for c in range(rangefirst, charcode + 1):
                    self.writeTest(c, ispunct)
                rangefirst = None
            else:
                if rangefirst != None:
                    raise SyntaxError
                if charcode >= breakat:
                    # we've exceeded our chunking size; break to a new file
                    self.startNewFile()
                    print "Break %s at %x for %x" % (self.filenum, breakat, charcode)
                    breakat = (floor(charcode / self.blocksize) + 1) * self.blocksize
                    print "Next break at %x" % breakat
                self.writeTest(charcode, ispunct)
        self.out.write(self.filefooter)

g = UnicodeTestGenerator()
g.write()
