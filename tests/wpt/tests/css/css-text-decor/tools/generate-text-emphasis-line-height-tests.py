#!/usr/bin/env python
# - * - coding: UTF-8 - * -

"""
This script generates tests text-emphasis-line-height-001 ~ 004 except
001z. They test the line height expansion in different directions. This
script outputs a list of all tests it generated in the format of Mozilla
reftest.list to the stdout.
"""

TEST_FILE = 'text-emphasis-line-height-{:03}{}.html'
TEST_TEMPLATE = '''<!DOCTYPE html>
<meta charset="utf-8">
<!-- This file was generated automatically by the script
     ./support/generate-text-emphasis-line-height-tests.py -->
<title>CSS Test: text-emphasis line height, {pos}, {wm}, {tag}</title>
<link rel="author" title="Xidorn Quan" href="https://www.upsuper.org">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<link rel="help" href="https://drafts.csswg.org/css-text-decor-3/#text-emphasis-position-property">
<meta name="assert" content="text emphasis marks should expand the line height like ruby if necessary">
<link rel="match" href="text-emphasis-line-height-{index:03}-ref.html">
<p>Pass if the emphasis marks are {dir} the black line:</p>
{start}試験テスト{end}
'''

REF_FILE = 'text-emphasis-line-height-{:03}-ref.html'
REF_TEMPLATE='''<!DOCTYPE html>
<meta charset="utf-8">
<!-- This file was generated automatically by the script
     ./support/generate-text-emphasis-line-height-tests.py -->
<title>CSS Reference: text-emphasis line height, {pos}</title>
<link rel="author" title="Xidorn Quan" href="https://www.upsuper.org">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<style> rt {{ font-variant-east-asian: inherit; }} </style>
<p>Pass if the emphasis marks are {dir} the black line:</p>
<div lang="ja" style="line-height: 1; border-{pos}: 1px solid black; writing-mode: {wm}; ruby-position: {posval}"><ruby>試<rt>&#x25CF;</rt>験<rt>&#x25CF;</rt>テ<rt>&#x25CF;</rt>ス<rt>&#x25CF;</rt>ト<rt>&#x25CF;</rt></ruby></div>
'''

STYLE1 = 'line-height: 1; border-{pos}: 1px solid black; ' + \
         'writing-mode: {wm}; text-emphasis-position: {posval};'
STYLE2 = 'text-emphasis: circle;'

TAGS = [
    # (tag, start, end)
    ('div', '<div lang="ja" style="{style1}{style2}">', '</div>'),
    ('span', '<div lang="ja" style="{style1}"><span style="{style2}">', '</span></div>'),
    ]
POSITIONS = [
    # pos, text-emphasis-position, ruby-position,
    #   writing-modes, dir text
    ('top', 'over right', 'over',
        ['horizontal-tb'], 'below'),
    ('bottom', 'under right', 'under',
        ['horizontal-tb'], 'over'),
    ('right', 'over right', 'over',
        ['vertical-rl', 'vertical-lr'], 'to the left of'),
    ('left', 'over left', 'under',
        ['vertical-rl', 'vertical-lr'], 'to the right of'),
    ]

import string

def write_file(filename, content):
    with open(filename, 'wb') as f:
        f.write(content.encode('UTF-8'))

print("# START tests from {}".format(__file__))
idx = 0
for (pos, emphasis_pos, ruby_pos, wms, dir) in POSITIONS:
    idx += 1
    ref_file = REF_FILE.format(idx)
    content = REF_TEMPLATE.format(pos=pos, dir=dir, wm=wms[0], posval=ruby_pos)
    write_file(ref_file, content)
    suffix = iter(string.ascii_lowercase)
    for wm in wms:
        style1 = STYLE1.format(pos=pos, wm=wm, posval=emphasis_pos)
        for (tag, start, end) in TAGS:
            test_file = TEST_FILE.format(idx, next(suffix))
            content = TEST_TEMPLATE.format(
                pos=pos, wm=wm, tag=tag, index=idx, dir=dir,
                start=start.format(style1=style1, style2=STYLE2), end=end)
            write_file(test_file, content)
            print("== {} {}".format(test_file, ref_file))
print("# END tests from {}".format(__file__))
