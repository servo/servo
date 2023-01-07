import pytest

from webdriver.client import Element, Frame, ShadowRoot, Window

from tests.support.asserts import assert_error, assert_success
from . import execute_script


def test_null(session):
    value = None
    result = execute_script(session, "return [arguments[0] === null, arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] is True
    assert actual[1] == value


@pytest.mark.parametrize("value, expected_type", [
    (True, "boolean"),
    (42, "number"),
    ("foo", "string"),
], ids=["boolean", "number", "string"])
def test_primitives(session, value, expected_type):
    result = execute_script(session, "return [typeof arguments[0], arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] == expected_type
    assert actual[1] == value


def test_collection(session):
    value = [1, 2, 3]
    result = execute_script(session, "return [Array.isArray(arguments[0]), arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] is True
    assert actual[1] == value


def test_object(session):
    value = {"foo": "bar", "cheese": 23}
    result = execute_script(session, "return [typeof arguments[0], arguments[0]]", args=[value])
    actual = assert_success(result)

    assert actual[0] == "object"
    assert actual[1] == value


@pytest.mark.parametrize("as_frame", [False, True], ids=["top_context", "child_context"])
def test_stale_element_reference(session, stale_element, as_frame):
    element = stale_element("<div>", "div", as_frame=as_frame)

    result = execute_script(session, "return 1;", args=[element])
    assert_error(result, "stale element reference")


@pytest.mark.parametrize("expression, expected_type, expected_class", [
    ("window.frames[0]", Frame, "Frame"),
    ("document.getElementById('foo')", Element, "HTMLDivElement"),
    ("document.getElementById('checkbox').shadowRoot", ShadowRoot, "ShadowRoot"),
    ("window", Window, "Window")
], ids=["frame", "node", "shadow-root", "window"])
def test_element_reference(session, iframe, inline, expression, expected_type, expected_class):
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

    result = execute_script(session, "return arguments[0].constructor.name", [reference])
    assert_success(result, expected_class)

