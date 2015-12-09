#!/usr/bin/env python
# - * - coding: UTF-8 - * -

"""
This script generates tests text-emphasis-ruby-001 ~ 004 which tests
emphasis marks with ruby in four directions. It outputs a list of all
tests it generated in the format of Mozilla reftest.list to the stdout.
"""

from __future__ import unicode_literals

TEST_FILE = 'text-emphasis-ruby-{:03}{}.html'
TEST_TEMPLATE = '''<!DOCTYPE html>
<meta charset="utf-8">
<title>CSS Test: text-emphasis and ruby, {wm}, {pos}</title>
<link rel="author" title="Xidorn Quan" href="https://www.upsuper.org">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<link rel="help" href="https://drafts.csswg.org/css-text-decor-3/#text-emphasis-position-property">
<meta name="assert" content="emphasis marks are drawn outside the ruby">
<link rel="match" href="reference/text-emphasis-ruby-{index:03}-ref.html">
<p>Pass if the emphasis marks are outside the ruby:</p>
<div style="line-height: 5; writing-mode: {wm}; ruby-position: {ruby_pos}; text-emphasis-position: {posval}">ルビ<span style="text-emphasis: circle">と<ruby>圏<rt>けん</rt>点<rt>てん</rt></ruby>を</span>同時</div>
'''

REF_FILE = 'text-emphasis-ruby-{:03}-ref.html'
REF_TEMPLATE = '''<!DOCTYPE html>
<meta charset="utf-8">
<title>CSS Reference: text-emphasis and ruby, {wm}, {pos}</title>
<link rel="author" title="Xidorn Quan" href="https://www.upsuper.org">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<style> rtc {{ font-variant-east-asian: inherit; }} </style>
<p>Pass if the emphasis marks are outside the ruby:</p>
<div style="line-height: 5; writing-mode: {wm}; ruby-position: {posval}">ルビ<ruby>と<rtc>&#x25CF;</rtc>圏<rt>けん</rt><rtc>&#x25CF;</rtc>点<rt>てん</rt><rtc>&#x25CF;</rtc>を<rtc>&#x25CF;</rtc></ruby>同時</div>
'''

TEST_CASES = [
        ('top', 'horizontal-tb', 'over', [
            ('horizontal-tb', 'over right')]),
        ('bottom', 'horizontal-tb', 'under', [
            ('horizontal-tb', 'under right')]),
        ('right', 'vertical-rl', 'over', [
            ('vertical-rl', 'over right'),
            ('vertical-lr', 'over right')]),
        ('left', 'vertical-rl', 'under', [
            ('vertical-rl', 'over left'),
            ('vertical-lr', 'over left')]),
    ]

SUFFIXES = ['', 'a']

def write_file(filename, content):
    with open(filename, 'wb') as f:
        f.write(content.encode('UTF-8'))

print("# START tests from {}".format(__file__))
idx = 0
for pos, ref_wm, ruby_pos, subtests in TEST_CASES:
    idx += 1
    ref_file = REF_FILE.format(idx)
    ref_content = REF_TEMPLATE.format(pos=pos, wm=ref_wm, posval=ruby_pos)
    write_file(ref_file, ref_content)
    suffix = iter(SUFFIXES)
    for wm, posval in subtests:
        test_file = TEST_FILE.format(idx, next(suffix))
        test_content = TEST_TEMPLATE.format(
            wm=wm, pos=pos, index=idx, ruby_pos=ruby_pos, posval=posval)
        write_file(test_file, test_content)
        print("== {} {}".format(test_file, ref_file))
print("# END tests from {}".format(__file__))
