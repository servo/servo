from __future__ import absolute_import, division, unicode_literals

import pytest

from html5lib import constants, parseFragment, serialize
from html5lib.filters import sanitizer


def sanitize_html(stream):
    parsed = parseFragment(stream)
    with pytest.deprecated_call():
        serialized = serialize(parsed,
                               sanitize=True,
                               omit_optional_tags=False,
                               use_trailing_solidus=True,
                               space_before_trailing_solidus=False,
                               quote_attr_values="always",
                               quote_char='"',
                               alphabetical_attributes=True)
    return serialized


def test_should_handle_astral_plane_characters():
    sanitized = sanitize_html("<p>&#x1d4b5; &#x1d538;</p>")
    expected = '<p>\U0001d4b5 \U0001d538</p>'
    assert expected == sanitized


def test_should_allow_relative_uris():
    sanitized = sanitize_html('<p><a href="/example.com"></a></p>')
    expected = '<p><a href="/example.com"></a></p>'
    assert expected == sanitized


def test_invalid_data_uri():
    sanitized = sanitize_html('<audio controls="" src="data:foobar"></audio>')
    expected = '<audio controls></audio>'
    assert expected == sanitized


def test_invalid_ipv6_url():
    sanitized = sanitize_html('<a href="h://]">')
    expected = "<a></a>"
    assert expected == sanitized


def test_data_uri_disallowed_type():
    sanitized = sanitize_html('<audio controls="" src="data:text/html,<html>"></audio>')
    expected = "<audio controls></audio>"
    assert expected == sanitized


def param_sanitizer():
    for ns, tag_name in sanitizer.allowed_elements:
        if ns != constants.namespaces["html"]:
            continue
        if tag_name in ['caption', 'col', 'colgroup', 'optgroup', 'option', 'table', 'tbody', 'td',
                        'tfoot', 'th', 'thead', 'tr', 'select']:
            continue  # TODO
        if tag_name == 'image':
            yield ("test_should_allow_%s_tag" % tag_name,
                   "<img title=\"1\"/>foo &lt;bad&gt;bar&lt;/bad&gt; baz",
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name))
        elif tag_name == 'br':
            yield ("test_should_allow_%s_tag" % tag_name,
                   "<br title=\"1\"/>foo &lt;bad&gt;bar&lt;/bad&gt; baz<br/>",
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name))
        elif tag_name in constants.voidElements:
            yield ("test_should_allow_%s_tag" % tag_name,
                   "<%s title=\"1\"/>foo &lt;bad&gt;bar&lt;/bad&gt; baz" % tag_name,
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name))
        else:
            yield ("test_should_allow_%s_tag" % tag_name,
                   "<%s title=\"1\">foo &lt;bad&gt;bar&lt;/bad&gt; baz</%s>" % (tag_name, tag_name),
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name))

    for ns, attribute_name in sanitizer.allowed_attributes:
        if ns is not None:
            continue
        if attribute_name != attribute_name.lower():
            continue  # TODO
        if attribute_name == 'style':
            continue
        attribute_value = 'foo'
        if attribute_name in sanitizer.attr_val_is_uri:
            attribute_value = '%s://sub.domain.tld/path/object.ext' % sanitizer.allowed_protocols[0]
        yield ("test_should_allow_%s_attribute" % attribute_name,
               "<p %s=\"%s\">foo &lt;bad&gt;bar&lt;/bad&gt; baz</p>" % (attribute_name, attribute_value),
               "<p %s='%s'>foo <bad>bar</bad> baz</p>" % (attribute_name, attribute_value))

    for protocol in sanitizer.allowed_protocols:
        rest_of_uri = '//sub.domain.tld/path/object.ext'
        if protocol == 'data':
            rest_of_uri = 'image/png;base64,aGVsbG8gd29ybGQ='
        yield ("test_should_allow_uppercase_%s_uris" % protocol,
               "<img src=\"%s:%s\">foo</a>" % (protocol, rest_of_uri),
               """<img src="%s:%s">foo</a>""" % (protocol, rest_of_uri))

    for protocol in sanitizer.allowed_protocols:
        rest_of_uri = '//sub.domain.tld/path/object.ext'
        if protocol == 'data':
            rest_of_uri = 'image/png;base64,aGVsbG8gd29ybGQ='
        protocol = protocol.upper()
        yield ("test_should_allow_uppercase_%s_uris" % protocol,
               "<img src=\"%s:%s\">foo</a>" % (protocol, rest_of_uri),
               """<img src="%s:%s">foo</a>""" % (protocol, rest_of_uri))


@pytest.mark.parametrize("expected, input",
                         (pytest.param(expected, input, id=id)
                          for id, expected, input in param_sanitizer()))
def test_sanitizer(expected, input):
    parsed = parseFragment(expected)
    expected = serialize(parsed,
                         omit_optional_tags=False,
                         use_trailing_solidus=True,
                         space_before_trailing_solidus=False,
                         quote_attr_values="always",
                         quote_char='"',
                         alphabetical_attributes=True)
    assert expected == sanitize_html(input)


def test_lowercase_color_codes_in_style():
    sanitized = sanitize_html("<p style=\"border: 1px solid #a2a2a2;\"></p>")
    expected = '<p style=\"border: 1px solid #a2a2a2;\"></p>'
    assert expected == sanitized


def test_uppercase_color_codes_in_style():
    sanitized = sanitize_html("<p style=\"border: 1px solid #A2A2A2;\"></p>")
    expected = '<p style=\"border: 1px solid #A2A2A2;\"></p>'
    assert expected == sanitized
