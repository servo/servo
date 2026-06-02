#!/usr/bin/env python
# - * - coding: UTF-8 - * -

"""
This script generates tests text-emphasis-style-property-011 ~ 020 which
cover all possible values of text-emphasis-style property, except none
and <string>, with horizontal writing mode. It outputs a list of all
tests it generated in the format of Mozilla reftest.list to the stdout.
"""

TEST_FILE = 'text-emphasis-style-property-{:03}{}.html'
TEST_TEMPLATE = '''<!DOCTYPE html>
<meta charset="utf-8">
<!-- This file was generated automatically by the script
     ./support/generate-text-emphasis-style-property-tests.py -->
<title>CSS Test: text-emphasis-style: {title}</title>
<link rel="author" title="Xidorn Quan" href="https://www.upsuper.org">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<link rel="help" href="https://drafts.csswg.org/css-text-decor-3/#text-emphasis-style-property">
<meta name="assert" content="'text-emphasis-style: {value}' produces {code} as emphasis marks.">
<link rel="match" href="text-emphasis-style-property-{index:03}-ref.html">
<p>Pass if there is a '{char}' above every character below:</p>
<div lang="ja" style="line-height: 5; text-emphasis-style: {value}">試験テスト</div>
'''

REF_FILE = 'text-emphasis-style-property-{:03}-ref.html'
REF_TEMPLATE = '''<!DOCTYPE html>
<meta charset="utf-8">
<!-- This file was generated automatically by the script
     ./support/generate-text-emphasis-style-property-tests.py -->
<title>CSS Reference: text-emphasis-style: {0}</title>
<link rel="author" title="Xidorn Quan" href="https://www.upsuper.org">
<link rel="author" title="Mozilla" href="https://www.mozilla.org">
<style> rt {{ font-variant-east-asian: inherit; }} </style>
<p>Pass if there is a '{1}' above every character below:</p>
<div lang="ja" style="line-height: 5;"><ruby>試<rt>{1}</rt>験<rt>{1}</rt>テ<rt>{1}</rt>ス<rt>{1}</rt>ト<rt>{1}</rt></ruby></div>
'''

DATA_SET = [
        ('dot',           0x2022, 0x25e6),
        ('circle',        0x25cf, 0x25cb),
        ('double-circle', 0x25c9, 0x25ce),
        ('triangle',      0x25b2, 0x25b3),
        ('sesame',        0xfe45, 0xfe46),
        ]

SUFFIXES = ['', 'a', 'b', 'c', 'd', 'e']

def get_html_entity(code):
    return '&#x{:04X};'.format(code)

def write_file(filename, content):
    with open(filename, 'wb') as f:
        f.write(content.encode('UTF-8'))

def write_test_file(idx, suffix, style, code, name=None):
    if not name:
        name = style
    filename = TEST_FILE.format(idx, suffix)
    write_file(filename, TEST_TEMPLATE.format(index=idx, value=style,
                                              char=get_html_entity(code),
                                              code='U+{:04X}'.format(code),
                                              title=name))
    print("== {} {}".format(filename, REF_FILE.format(idx)))

idx = 10
def write_files(style, code):
    global idx
    idx += 1
    fill, shape = style
    basic_style = "{} {}".format(fill, shape)
    write_file(REF_FILE.format(idx),
               REF_TEMPLATE.format(basic_style, get_html_entity(code)))
    suffix = iter(SUFFIXES)
    write_test_file(idx, next(suffix), basic_style, code)
    write_test_file(idx, next(suffix), "{} {}".format(shape, fill), code)
    if fill == 'filled':
        write_test_file(idx, next(suffix), shape, code)
    if shape == 'circle':
        write_test_file(idx, next(suffix), fill, code, fill + ', horizontal')

print("# START tests from {}".format(__file__))
for name, code, _ in DATA_SET:
    write_files(('filled', name), code)
for name, _, code in DATA_SET:
    write_files(('open', name), code)
print("# END tests from {}".format(__file__))
