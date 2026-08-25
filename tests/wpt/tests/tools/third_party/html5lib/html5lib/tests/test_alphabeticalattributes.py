from __future__ import absolute_import, division, unicode_literals

from collections import OrderedDict

import pytest

import html5lib
from html5lib.filters.alphabeticalattributes import Filter
from html5lib.serializer import HTMLSerializer


@pytest.mark.parametrize('msg, attrs, expected_attrs', [
    (
        'no attrs',
        {},
        {}
    ),
    (
        'one attr',
        {(None, 'alt'): 'image'},
        OrderedDict([((None, 'alt'), 'image')])
    ),
    (
        'multiple attrs',
        {
            (None, 'src'): 'foo',
            (None, 'alt'): 'image',
            (None, 'style'): 'border: 1px solid black;'
        },
        OrderedDict([
            ((None, 'alt'), 'image'),
            ((None, 'src'), 'foo'),
            ((None, 'style'), 'border: 1px solid black;')
        ])
    ),
])
def test_alphabetizing(msg, attrs, expected_attrs):
    tokens = [{'type': 'StartTag', 'name': 'img', 'data': attrs}]
    output_tokens = list(Filter(tokens))

    attrs = output_tokens[0]['data']
    assert attrs == expected_attrs


def test_with_different_namespaces():
    tokens = [{
        'type': 'StartTag',
        'name': 'pattern',
        'data': {
            (None, 'id'): 'patt1',
            ('http://www.w3.org/1999/xlink', 'href'): '#patt2'
        }
    }]
    output_tokens = list(Filter(tokens))

    attrs = output_tokens[0]['data']
    assert attrs == OrderedDict([
        ((None, 'id'), 'patt1'),
        (('http://www.w3.org/1999/xlink', 'href'), '#patt2')
    ])


def test_with_serializer():
    """Verify filter works in the context of everything else"""
    parser = html5lib.HTMLParser()
    dom = parser.parseFragment('<svg><pattern xlink:href="#patt2" id="patt1"></svg>')
    walker = html5lib.getTreeWalker('etree')
    ser = HTMLSerializer(
        alphabetical_attributes=True,
        quote_attr_values='always'
    )

    # FIXME(willkg): The "xlink" namespace gets dropped by the serializer. When
    # that gets fixed, we can fix this expected result.
    assert (
        ser.render(walker(dom)) ==
        '<svg><pattern id="patt1" href="#patt2"></pattern></svg>'
    )
