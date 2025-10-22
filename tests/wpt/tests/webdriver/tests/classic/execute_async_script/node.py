import pytest

from webdriver.client import ShadowRoot, WebElement

from tests.support.asserts import assert_error, assert_success
from . import execute_async_script


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

# https://dom.spec.whatwg.org/#ref-for-dom-node-nodetype%E2%91%A0
NODE_TYPE = {
    "element": 1,
    "attribute": 2,
    "text": 3,
    "cdata": 4,
    "processing_instruction": 7,
    "comment": 8,
    "document": 9,
    "doctype": 10,
}

@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_detached_shadow_root(session, get_test_page, as_frame):
    session.url = get_test_page(as_frame)

    if as_frame:
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)

    element = session.find.css("custom-element", all=False)

    # Retrieve shadow root to add it to the node cache
    shadow_root = element.shadow_root

    result = execute_async_script(session, """
        const [elem, shadowRoot, resolve] = arguments;
        elem.remove();
        resolve(shadowRoot);
        """, args=[element, shadow_root])
    assert_error(result, "detached shadow root")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element(session, get_test_page, as_frame):
    session.url = get_test_page(as_frame=as_frame)

    if as_frame:
        frame = session.find.css("iframe", all=False)
        session.switch_frame(frame)

    element = session.find.css("div", all=False)

    result = execute_async_script(session, """
        const [elem, resolve] = arguments;
        elem.remove();
        resolve(elem);
        """, args=[element])
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("expression, expected_type", [
    (""" document.querySelector("svg").attributes[0] """, "attribute"),
    (""" document.querySelector("div#text-node").childNodes[1] """, "text"),
    (""" document.implementation.createDocument("", "root", null).createCDATASection("foo") """, "cdata"),
    (""" document.createProcessingInstruction("xml-stylesheet", "href='foo.css'") """, "processing_instruction"),
    (""" document.querySelector("div#comment").childNodes[0] """, "comment"),
    (""" document""", "document"),
    (""" document.doctype""", "doctype"),
], ids=["attribute", "text", "cdata", "processing_instruction", "comment", "document", "doctype"])
def test_node_type(session, inline, expression, expected_type):
    session.url = inline(PAGE_DATA)

    response = execute_async_script(
        session,
        f"""
    const [resolve] = arguments;
    const result = {expression};
    resolve({{
        'result': result,
        'type': result.nodeType
    }});""",
    )
    result = assert_success(response)

    assert result["type"] == NODE_TYPE[expected_type]

    if expected_type == "document":
        assert 'location' in result['result']
    else:
        assert result['result'] == {}



@pytest.mark.parametrize("expression, expected_type", [
    ("document.querySelector('div')", WebElement),
    ("document.querySelector('custom-element').shadowRoot", ShadowRoot),
], ids=["element", "shadow-root"])
def test_element_reference(session, get_test_page, expression, expected_type):
    session.url = get_test_page()

    result = execute_async_script(session, f"arguments[0]({expression})")
    reference = assert_success(result)
    assert isinstance(reference, expected_type)

