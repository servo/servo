from __future__ import absolute_import, division, unicode_literals

import sys
import os

if __name__ == '__main__':
    # Allow us to import from the src directory
    os.chdir(os.path.split(os.path.abspath(__file__))[0])
    sys.path.insert(0, os.path.abspath(os.path.join(os.pardir, "src")))

from html5lib.tokenizer import HTMLTokenizer


class HTMLParser(object):
    """ Fake parser to test tokenizer output """
    def parse(self, stream, output=True):
        tokenizer = HTMLTokenizer(stream)
        for token in tokenizer:
            if output:
                print(token)

if __name__ == "__main__":
    x = HTMLParser()
    if len(sys.argv) > 1:
        if len(sys.argv) > 2:
            import hotshot
            import hotshot.stats
            prof = hotshot.Profile('stats.prof')
            prof.runcall(x.parse, sys.argv[1], False)
            prof.close()
            stats = hotshot.stats.load('stats.prof')
            stats.strip_dirs()
            stats.sort_stats('time')
            stats.print_stats()
        else:
            x.parse(sys.argv[1])
    else:
        print("""Usage: python mockParser.py filename [stats]
        If stats is specified the hotshots profiler will run and output the
        stats instead.
        """)
