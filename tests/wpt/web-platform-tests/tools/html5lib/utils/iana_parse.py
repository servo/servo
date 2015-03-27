#!/usr/bin/env python
import sys
import urllib.request, urllib.error, urllib.parse
import codecs

def main():
    encodings = []
    f = urllib.request.urlopen(sys.argv[1])
    for line in f:
        if line.startswith("Name: ") or line.startswith("Alias: "):
            enc = line.split()[1]
            try:
                codecs.lookup(enc)
                if enc.lower not in encodings:
                    encodings.append(enc.lower())
            except LookupError:
                pass
    sys.stdout.write("encodings = frozenset((\n")
    for enc in encodings:
        sys.stdout.write('    "%s",\n'%enc)
    sys.stdout.write('    ))')

if __name__ == "__main__":
    main()