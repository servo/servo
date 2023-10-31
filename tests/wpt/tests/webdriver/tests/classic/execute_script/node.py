import pytest

from webdriver.client import WebElement, ShadowRoot
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
def test_detached_shadow_root(session, get_test_page, as_frame):
    session.url = get_test_page(as_frame)

    if as_frame:
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)

    element = session.find.css("custom-element", all=False)

    # Retrieve shadow root to add it to the node cache
    shadow_root = element.shadow_root

    result = execute_script(session, """
        const [elem, shadowRoot] = arguments;
        elem.remove();
        return shadowRoot;
        """, args=[element, shadow_root])
    assert_error(result, "detached shadow root")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element(session, get_test_page, as_frame):
    session.url = get_test_page(as_frame)

    if as_frame:
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)

    element = session.find.css("div", all=False)

    result = execute_script(session, """
        const elem = arguments[0];
        elem.remove();
        return elem;
        """, args=[element])
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("expression, expected_type", [
    ("document.querySelector('div')", WebElement),
    ("document.querySelector('custom-element').shadowRoot", ShadowRoot),
], ids=["element", "shadow-root"])
def test_web_reference(session, get_test_page, expression, expected_type):
    session.url = get_test_page()

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
def test_not_supported_nodes(session, inline, expression):
    session.url = inline(PAGE_DATA)

    result = execute_script(session, f"return {expression}")
    assert_error(result, "javascript error")
