import pytest

from webdriver.client import Element, Frame, ShadowRoot, Window
from tests.support.asserts import assert_error, assert_success
from . import execute_script


PAGE_DATA = """
    <div id="deep"><p><span></span></p><br/></div>
    <div id="text-node"><p></p>Lorem</div>
    <br/>
    <svg id="foo"></svg>
    <div id="comment"><!-- Comment --></div>
    <script>
        var svg = document.querySelector("svg");
        svg.setAttributeNS("http://www.w3.org/2000/svg", "svg:foo", "bar");
    </script>
"""


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, iframe, inline, as_frame):
    if as_frame:
        session.url = inline(iframe("<div>"))
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)
    else:
        session.url = inline("<div>")

    element = session.find.css("div", all=False)

    result = execute_script(session, """
        const elem = arguments[0];
        elem.remove();
        return elem;
        """, args=[element])
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("expression, expected_type", [
    ("window.frames[0]", Frame),
    ("document.getElementById('foo')", Element),
    ("document.getElementById('checkbox').shadowRoot", ShadowRoot),
    ("window", Window),
], ids=["frame", "node", "shadow-root", "window"])
def test_element_reference(session, iframe, inline, expression, expected_type):
    session.url = inline(f"""
        <style>
            custom-checkbox-element {{
                display:block; width:20px; height:20px;
            }}
        </style>
        <custom-checkbox-element id='checkbox'></custom-checkbox-element>
        <script>
            customElements.define('custom-checkbox-element',
                class extends HTMLElement {{
                    constructor() {{
                        super();
                        this.attachShadow({{mode: 'open'}}).innerHTML = `
                            <div><input type="checkbox"/></div>
                        `;
                    }}
                }});
        </script>
        <div id="foo"/>
        {iframe("<p>")}""")

    result = execute_script(session, f"return {expression}")
    reference = assert_success(result)
    assert isinstance(reference, expected_type)


@pytest.mark.parametrize("expression", [
    (""" document.querySelector("svg").attributes[0] """),
    (""" document.querySelector("div#text-node").childNodes[1] """),
    (""" document.querySelector("foo").childNodes[1] """),
    (""" document.createProcessingInstruction("xml-stylesheet", "href='foo.css'") """),
    (""" document.querySelector("div#comment").childNodes[0] """),
    (""" document"""),
    (""" document.doctype"""),
], ids=["attribute", "text", "cdata", "processing_instruction", "comment", "document", "doctype"])
def test_non_element_nodes(session, inline, expression):
    session.url = inline(PAGE_DATA)

    result = execute_script(session, f"return {expression}")
    assert_error(result, "javascript error")
