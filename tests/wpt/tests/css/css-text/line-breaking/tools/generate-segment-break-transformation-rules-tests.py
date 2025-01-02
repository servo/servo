#!/usr/bin/env python
# - * - coding: UTF-8 - * -

"""
This script generates tests segment-break-transformation-rules-001 ~ 049 which
cover all possible combinations of characters at two sides of segment breaks.
More specifically, there are seven types of characters involve in these rules:

1. East Asian Full-width (F)
2. East Asian Half-width (H)
3. East Asian Wide (W) except Hangul
4. East Asian Narrow (Na)
5. East Asian Ambiguous (A)
6. Not East Asian (Neutral)
7. Hangul

So there are 49 different combinations. It outputs a list of all
tests it generated in the format of Mozilla reftest.list to the stdout.
"""

TEST_FILE = 'segment-break-transformation-rules-{:03}.html'
TEST_TEMPLATE = '''<!DOCTYPE html>
<meta charset="utf-8">
<title>CSS Reftest Test: Segment Break Transformation Rules</title>
<link rel="author" title="Chun-Min (Jeremy) Chen" href="mailto:jeremychen@mozilla.com">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<link rel="help" href="https://drafts.csswg.org/css-text-3/#line-break-transform">
<meta name="assert" content="'segment-break-transformation-rules: with {prev}/{next} in front/back of the semgment break.">
<link rel="stylesheet" type="text/css" href="/fonts/ahem.css">
<link rel="match" href="segment-break-transformation-rules-{index:03}-ref.html">
<style> p {{ font-family: ahem; }} </style>
<div>Pass if there is {expect} white space between the two strings below.
<p>{prevchar}&#x000a;{nextchar}</p>
</div>
'''

REF_FILE = 'segment-break-transformation-rules-{:03}-ref.html'
REF_TEMPLATE_REMOVE = '''<!DOCTYPE html>
<meta charset="utf-8">
<title>CSS Reftest Reference: Segment Break Transformation Rules</title>
<link rel="author" title="Chun-Min (Jeremy) Chen" href="mailto:jeremychen@mozilla.com">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<link rel="stylesheet" type="text/css" href="/fonts/ahem.css">
<style> p {{ font-family: ahem; }} </style>
<div>Pass if there is NO white space between the two strings below.
<p>{0}{1}</p>
</div>
'''
REF_TEMPLATE_KEEP = '''<!DOCTYPE html>
<meta charset="utf-8">
<title>CSS Reftest Reference: Segment Break Transformation Rules</title>
<link rel="author" title="Chun-Min (Jeremy) Chen" href="mailto:jeremychen@mozilla.com">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<link rel="stylesheet" type="text/css" href="/fonts/ahem.css">
<style> p {{ font-family: ahem; }} </style>
<div>Pass if there is ONE white space between the two strings below.
<p>{0}{2}{1}</p>
</div>
'''

CHAR_SET = [
        ('East Asian Full-width (F)',         'ＦＵＬＬＷＩＤＴＨ'),
        ('East Asian Half-width (H)',         'ﾃｽﾄ'),
        ('East Asian Wide (W) except Hangul', '測試'),
        ('East Asian Narrow (Na)',            'narrow'),
        ('East Asian Ambiguous (A)',          '■'),
        ('Not East Asian (Neutral)',          'آزمون'),
        ('Hangul',                            '테스트'),
        ]

def write_file(filename, content):
    with open(filename, 'wb') as f:
        f.write(content.encode('UTF-8'))

print("# START tests from {}".format(__file__))
global idx
idx = 0
for i, (prevtype, prevchars) in enumerate(CHAR_SET):
    for j, (nextype, nextchars) in enumerate(CHAR_SET):
        idx += 1
        reffilename = REF_FILE.format(idx)
        testfilename = TEST_FILE.format(idx)
        # According to CSS Text 3 - 4.1.2. Segment Break Transformation Rules,
        # if the East Asian Width property of both the character before and
        # after the segment break is F, W, or H (not A), and neither side is
        # Hangul, then the segment break is removed. Otherwise, the segment
        # break is converted to a space (U+0020).
        if i < 3 and j < 3:
            write_file(reffilename,
                       REF_TEMPLATE_REMOVE.format(prevchars, nextchars))
            write_file(testfilename,
                       TEST_TEMPLATE.format(index=idx, prev=prevtype,
                                            next=nextype,
                                            prevchar=prevchars,
                                            nextchar=nextchars,
                                            expect='NO'))
        else:
            write_file(reffilename,
                       REF_TEMPLATE_KEEP.format(prevchars, nextchars, '&#x0020;'))
            write_file(testfilename,
                       TEST_TEMPLATE.format(index=idx, prev=prevtype,
                                            next=nextype,
                                            prevchar=prevchars,
                                            nextchar=nextchars,
                                            expect='ONE'))
        print("== {} {}".format(testfilename, reffilename))
print("# END tests from {}".format(__file__))
