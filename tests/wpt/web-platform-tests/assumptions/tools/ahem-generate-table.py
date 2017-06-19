from __future__ import print_function, unicode_literals

import itertools
import unicodedata

from fontTools.ttLib import TTFont

try:
    chr(0x100)
except ValueError:
    chr = unichr

def grouper(n, iterable):
    """
    >>> list(grouper(3, 'ABCDEFG'))
    [['A', 'B', 'C'], ['D', 'E', 'F'], ['G']]
    """
    iterable = iter(iterable)
    return iter(lambda: list(itertools.islice(iterable, n)), [])

ttf = TTFont("../../css/fonts/ahem/ahem.ttf")

chars = {char for table in ttf['cmap'].tables for char in table.cmap.keys()}

# exclude chars that can't be represented as HTML numeric character refs
chars = chars - (set(range(0x80, 0x9F+1)) | {0x00})

chars_sorted = sorted(chars)

per_row = 17


def build_header(is_test):
    rv = []

    rv.append("""
<!doctype html>
<title>Ahem checker</title>""")

    if is_test:
        rv.append("""
<link rel="match" href="ahem-ref.html">""")

    rv.append("""
<style>""")

    if not is_test:
        rv.append("""
@font-face {
  font-family: Ahem;
  src: url("../css/fonts/ahem/ahem.ttf");
}""")

    rv.append("""
* {
  padding: 0;
  margin: 0;
  border: none;
}

table {
  font: 15px/1 Ahem;
  border-collapse: separate;
  border-spacing: 1px;
  table-layout: fixed;
}

td {
  width: 34px;
}
</style>
""")

    return "".join(rv)


def build_table():
    rv = []

    rv.append("<table>\n")
    for row in grouper(per_row, chars_sorted):
        rv.append(" " * 4 + "<tr>\n")
        for codepoint in row:
            assert codepoint <= 0xFFFF
            try:
                name = unicodedata.name(chr(codepoint))
            except ValueError:
                rv.append(" " * 8 + "<td>&#x%04X;x <!-- U+%04X -->\n" % (codepoint, codepoint))
            else:
                rv.append(" " * 8 + "<td>&#x%04X;x <!-- U+%04X: %s -->\n" % (codepoint, codepoint, name))
    rv.append("</table>\n")

    return "".join(rv)


with open("../ahem.html", "w") as f1:
    f1.write(build_header(True))
    f1.write(build_table())

with open("../ahem-ref.html", "w") as f1:
    f1.write(build_header(False))
    f1.write(build_table())
