import pytest
from webdriver.client import Element, Frame, ShadowRoot, Window

from tests.support.asserts import assert_error, assert_success
from . import execute_script


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference_as_argument(session, stale_element, as_frame):
    element = stale_element("<div>", "div", as_frame=as_frame)

    result = execute_script(session, "return 1;", args=[element])
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference_as_returned_value(session, iframe, inline, as_frame):
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


@pytest.mark.parametrize("expression, type, name", [
    ("window.frames[0]", Frame, "Frame"),
    ("document.getElementById('foo')", Element, "HTMLDivElement"),
    ("document.getElementById('checkbox').shadowRoot", ShadowRoot, "ShadowRoot"),
    ("window", Window, "Window")
], ids=["frame", "node", "shadow-root", "window"])
def test_element_reference(session, iframe, inline, expression, type, name):
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
    assert isinstance(reference, type)

    result = execute_script(session, "return arguments[0].constructor.name", [reference])
    name = assert_success(result, name)


def test_document_as_object(session, inline):
    session.url = inline("")

    # Retrieving the HTMLDocument is not possible due to cyclic references
    result = execute_script(session, "arguments[0](document)")
    assert_error(result, "javascript error")
