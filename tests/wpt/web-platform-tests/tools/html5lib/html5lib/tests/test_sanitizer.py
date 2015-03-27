from __future__ import absolute_import, division, unicode_literals

try:
    import json
except ImportError:
    import simplejson as json

from html5lib import html5parser, sanitizer, constants, treebuilders


def toxmlFactory():
    tree = treebuilders.getTreeBuilder("etree")

    def toxml(element):
        # encode/decode roundtrip required for Python 2.6 compatibility
        result_bytes = tree.implementation.tostring(element, encoding="utf-8")
        return result_bytes.decode("utf-8")

    return toxml


def runSanitizerTest(name, expected, input, toxml=None):
    if toxml is None:
        toxml = toxmlFactory()
    expected = ''.join([toxml(token) for token in html5parser.HTMLParser().
                        parseFragment(expected)])
    expected = json.loads(json.dumps(expected))
    assert expected == sanitize_html(input)


def sanitize_html(stream, toxml=None):
    if toxml is None:
        toxml = toxmlFactory()
    return ''.join([toxml(token) for token in
                    html5parser.HTMLParser(tokenizer=sanitizer.HTMLSanitizer).
                    parseFragment(stream)])


def test_should_handle_astral_plane_characters():
    assert '<html:p xmlns:html="http://www.w3.org/1999/xhtml">\U0001d4b5 \U0001d538</html:p>' == sanitize_html("<p>&#x1d4b5; &#x1d538;</p>")


def test_sanitizer():
    toxml = toxmlFactory()
    for tag_name in sanitizer.HTMLSanitizer.allowed_elements:
        if tag_name in ['caption', 'col', 'colgroup', 'optgroup', 'option', 'table', 'tbody', 'td', 'tfoot', 'th', 'thead', 'tr']:
            continue  # TODO
        if tag_name != tag_name.lower():
            continue  # TODO
        if tag_name == 'image':
            yield (runSanitizerTest, "test_should_allow_%s_tag" % tag_name,
                   "<img title=\"1\"/>foo &lt;bad&gt;bar&lt;/bad&gt; baz",
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name),
                   toxml)
        elif tag_name == 'br':
            yield (runSanitizerTest, "test_should_allow_%s_tag" % tag_name,
                   "<br title=\"1\"/>foo &lt;bad&gt;bar&lt;/bad&gt; baz<br/>",
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name),
                   toxml)
        elif tag_name in constants.voidElements:
            yield (runSanitizerTest, "test_should_allow_%s_tag" % tag_name,
                   "<%s title=\"1\"/>foo &lt;bad&gt;bar&lt;/bad&gt; baz" % tag_name,
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name),
                   toxml)
        else:
            yield (runSanitizerTest, "test_should_allow_%s_tag" % tag_name,
                   "<%s title=\"1\">foo &lt;bad&gt;bar&lt;/bad&gt; baz</%s>" % (tag_name, tag_name),
                   "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name),
                   toxml)

    for tag_name in sanitizer.HTMLSanitizer.allowed_elements:
        tag_name = tag_name.upper()
        yield (runSanitizerTest, "test_should_forbid_%s_tag" % tag_name,
               "&lt;%s title=\"1\"&gt;foo &lt;bad&gt;bar&lt;/bad&gt; baz&lt;/%s&gt;" % (tag_name, tag_name),
               "<%s title='1'>foo <bad>bar</bad> baz</%s>" % (tag_name, tag_name),
               toxml)

    for attribute_name in sanitizer.HTMLSanitizer.allowed_attributes:
        if attribute_name != attribute_name.lower():
            continue  # TODO
        if attribute_name == 'style':
            continue
        yield (runSanitizerTest, "test_should_allow_%s_attribute" % attribute_name,
               "<p %s=\"foo\">foo &lt;bad&gt;bar&lt;/bad&gt; baz</p>" % attribute_name,
               "<p %s='foo'>foo <bad>bar</bad> baz</p>" % attribute_name,
               toxml)

    for attribute_name in sanitizer.HTMLSanitizer.allowed_attributes:
        attribute_name = attribute_name.upper()
        yield (runSanitizerTest, "test_should_forbid_%s_attribute" % attribute_name,
               "<p>foo &lt;bad&gt;bar&lt;/bad&gt; baz</p>",
               "<p %s='display: none;'>foo <bad>bar</bad> baz</p>" % attribute_name,
               toxml)

    for protocol in sanitizer.HTMLSanitizer.allowed_protocols:
        yield (runSanitizerTest, "test_should_allow_%s_uris" % protocol,
               "<a href=\"%s\">foo</a>" % protocol,
               """<a href="%s">foo</a>""" % protocol,
               toxml)

    for protocol in sanitizer.HTMLSanitizer.allowed_protocols:
        yield (runSanitizerTest, "test_should_allow_uppercase_%s_uris" % protocol,
               "<a href=\"%s\">foo</a>" % protocol,
               """<a href="%s">foo</a>""" % protocol,
               toxml)
