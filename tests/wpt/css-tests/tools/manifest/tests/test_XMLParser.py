from xml.etree.ElementTree import ParseError

import pytest

from ..XMLParser import XMLParser


@pytest.mark.parametrize("s", [
    '<foo>&nbsp;</foo>',
    '<!DOCTYPE foo><foo>&nbsp;</foo>',
    '<!DOCTYPE foo PUBLIC "fake" "id"><foo>&nbsp;</foo>',
    '<!DOCTYPE foo PUBLIC "fake" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"><foo>&nbsp;</foo>',
    '<!DOCTYPE foo PUBLIC "fake-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"><foo>&nbsp;</foo>'
])
def test_undefined_entity(s):
    with pytest.raises(ParseError):
        p = XMLParser()
        p.feed(s)
        p.close()


@pytest.mark.parametrize("s", [
    '<!DOCTYPE foo PUBLIC "-//W3C//DTD XHTML 1.0 Strict//EN" "http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"><foo>&nbsp;</foo>'
])
def test_defined_entity(s):
    p = XMLParser()
    p.feed(s)
    d = p.close()
    assert d.tag == u"foo"
    assert d.text == u"\u00A0"


def test_pi():
    p = XMLParser()
    p.feed('<foo><?foo bar?></foo>')
    d = p.close()
    assert d.tag == u"foo"
    assert len(d) == 0


def test_comment():
    p = XMLParser()
    p.feed('<foo><!-- data --></foo>')
    d = p.close()
    assert d.tag == u"foo"
    assert len(d) == 0


def test_unsupported_encoding():
    p = XMLParser()
    p.feed(u"<?xml version='1.0' encoding='Shift-JIS'?><foo>\u3044</foo>".encode("shift-jis"))
    d = p.close()
    assert d.tag == u"foo"
    assert d.text == u"\u3044"
