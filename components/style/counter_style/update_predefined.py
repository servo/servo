#!/usr/bin/env python

import os.path
import re
import urllib

def main(filename):
    names = [
        re.search(">([^>]+)</dfn>", line).group(1)
        for line in urllib.urlopen("https://drafts.csswg.org/css-counter-styles/")
        if 'data-dfn-for="<counter-style-name>"' in line
    ]
    with open(filename, "wb") as f:
        f.write("predefined! {\n")
        for name in names:
            # FIXME https://github.com/w3c/csswg-drafts/issues/1285
            if name == 'decimal':
                continue
            f.write('    "%s",\n' % name)
        f.write('}\n')

if __name__ == "__main__":
    main(os.path.join(os.path.dirname(__file__), "predefined.rs"))
