#!/usr/bin/env python

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/. */

import os.path
import re
import urllib


def main(filename):
    names = [
        re.search('>([^>]+)(</dfn>|<a class="self-link")', line).group(1)
        for line in urllib.urlopen("https://drafts.csswg.org/css-counter-styles/")
        if 'data-dfn-for="<counter-style-name>"' in line
        or 'data-dfn-for="<counter-style>"' in line
    ]
    with open(filename, "wb") as f:
        f.write("""\
/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

predefined! {
""")
        for name in names:
            f.write('    "%s",\n' % name)
        f.write('}\n')


if __name__ == "__main__":
    main(os.path.join(os.path.dirname(__file__), "predefined.rs"))
