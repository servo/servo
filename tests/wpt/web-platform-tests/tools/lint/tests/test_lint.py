from __future__ import unicode_literals

from ..lint import filter_whitelist_errors, parse_whitelist
import six

def test_lint():
    filtered = filter_whitelist_errors({}, '', [])
    assert filtered == []


def test_parse_whitelist():
    input_buffer = six.StringIO("""
# Comment
CR AT EOL: svg/import/*
CR AT EOL: streams/resources/test-utils.js

INDENT TABS: .gitmodules
INDENT TABS: app-uri/*
INDENT TABS: svg/*

TRAILING WHITESPACE: app-uri/*

CONSOLE:streams/resources/test-utils.js: 12

*:*.pdf
*:resources/*
""")

    expected = {
        '*.pdf': {
            '*': {None},
        },
        '.gitmodules': {
            'INDENT TABS': {None},
        },
        'app-uri/*': {
            'TRAILING WHITESPACE': {None},
            'INDENT TABS': {None},
        },
        'resources/*': {
            '*': {None},
        },
        'streams/resources/test-utils.js': {
            'CONSOLE': {12},
            'CR AT EOL': {None},
        },
        'svg/*': {
            'INDENT TABS': {None},
        },
        'svg/import/*': {
            'CR AT EOL': {None},
        },
    }
    assert parse_whitelist(input_buffer) == expected
