
import py
from py._xmlgen import unicode, html, raw
import sys

class ns(py.xml.Namespace):
    pass

def test_escape():
    uvalue = py.builtin._totext('\xc4\x85\xc4\x87\n\xe2\x82\xac\n', 'utf-8')
    class A:
        def __unicode__(self):
            return uvalue
        def __str__(self):
            x = self.__unicode__()
            if sys.version_info[0] < 3:
                return x.encode('utf-8')
            return x
    y = py.xml.escape(uvalue)
    assert y == uvalue
    x = py.xml.escape(A())
    assert x == uvalue
    if sys.version_info[0] < 3:
        assert isinstance(x, unicode)
        assert isinstance(y, unicode)
        y = py.xml.escape(uvalue.encode('utf-8'))
        assert y == uvalue


def test_tag_with_text():
    x = ns.hello("world")
    u = unicode(x)
    assert u == "<hello>world</hello>"

def test_class_identity():
    assert ns.hello is ns.hello

def test_tag_with_text_and_attributes():
    x = ns.some(name="hello", value="world")
    assert x.attr.name == 'hello'
    assert x.attr.value == 'world'
    u = unicode(x)
    assert u == '<some name="hello" value="world"/>'

def test_tag_with_subclassed_attr_simple():
    class my(ns.hello):
        class Attr(ns.hello.Attr):
            hello="world"
    x = my()
    assert x.attr.hello == 'world'
    assert unicode(x) == '<my hello="world"/>'

def test_tag_with_raw_attr():
    x = html.object(data=raw('&'))
    assert unicode(x) == '<object data="&"></object>'

def test_tag_nested():
    x = ns.hello(ns.world())
    unicode(x) # triggers parentifying
    assert x[0].parent is x
    u = unicode(x)
    assert u == '<hello><world/></hello>'

def test_list_nested():
    x = ns.hello([ns.world()]) #pass in a list here
    u = unicode(x)
    assert u == '<hello><world/></hello>'

def test_tag_xmlname():
    class my(ns.hello):
        xmlname = 'world'
    u = unicode(my())
    assert u == '<world/>'

def test_tag_with_text_entity():
    x = ns.hello('world & rest')
    u = unicode(x)
    assert u == "<hello>world &amp; rest</hello>"

def test_tag_with_text_and_attributes_entity():
    x = ns.some(name="hello & world")
    assert x.attr.name == "hello & world"
    u = unicode(x)
    assert u == '<some name="hello &amp; world"/>'

def test_raw():
    x = ns.some(py.xml.raw("<p>literal</p>"))
    u = unicode(x)
    assert u == "<some><p>literal</p></some>"


def test_html_name_stickyness():
    class my(html.p):
        pass
    x = my("hello")
    assert unicode(x) == '<p>hello</p>'

def test_stylenames():
    class my:
        class body(html.body):
            style = html.Style(font_size = "12pt")
    u = unicode(my.body())
    assert u == '<body style="font-size: 12pt"></body>'

def test_class_None():
    t = html.body(class_=None)
    u = unicode(t)
    assert u == '<body></body>'

def test_alternating_style():
    alternating = (
        html.Style(background="white"),
        html.Style(background="grey"),
    )
    class my(html):
        class li(html.li):
            def style(self):
                i = self.parent.index(self)
                return alternating[i%2]
            style = property(style)

    x = my.ul(
            my.li("hello"),
            my.li("world"),
            my.li("42"))
    u = unicode(x)
    assert u == ('<ul><li style="background: white">hello</li>'
                     '<li style="background: grey">world</li>'
                     '<li style="background: white">42</li>'
                 '</ul>')

def test_singleton():
    h = html.head(html.link(href="foo"))
    assert unicode(h) == '<head><link href="foo"/></head>'

    h = html.head(html.script(src="foo"))
    assert unicode(h) == '<head><script src="foo"></script></head>'

def test_inline():
    h = html.div(html.span('foo'), html.span('bar'))
    assert (h.unicode(indent=2) ==
            '<div><span>foo</span><span>bar</span></div>')

def test_object_tags():
    o = html.object(html.object())
    assert o.unicode(indent=0) == '<object><object></object></object>'
