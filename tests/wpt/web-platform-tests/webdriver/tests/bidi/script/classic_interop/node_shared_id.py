import pytest

from webdriver import Element, ShadowRoot
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "expression, expected_node_type",
    [
        ("""return document.querySelector("div#with-children")""", 1),
        ("""return document.querySelector("custom-element").shadowRoot""", 11),
    ],
    ids=["Element", "ShadowRoot"],
)
async def test_web_reference_created_in_classic(
    bidi_session,
    current_session,
    get_test_page,
    top_context,
    expression,
    expected_node_type
):
    current_session.url = get_test_page()

    node = current_session.execute_script(expression)
    shared_id = node.id

    # Use element reference from WebDriver classic in WebDriver BiDi
    result = await bidi_session.script.call_function(
        function_declaration="(node)=>{return node.nodeType}",
        arguments=[{"sharedId": shared_id}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "number", "value": expected_node_type}


@pytest.mark.parametrize(
    "expression, expected",
    [
        ("""document.querySelector("div#with-children")""", 1),
        ("""document.querySelector("custom-element").shadowRoot""", 11),
    ],
    ids=["Element", "ShadowRoot"],
)
async def test_web_reference_created_in_bidi(
    bidi_session,
    current_session,
    get_test_page,
    top_context,
    expression,
    expected
):
    current_session.url = get_test_page()

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    nodeType = result["value"]["nodeType"]
    assert nodeType == expected

    # Use web reference from WebDriver BiDi in WebDriver classic
    types = {1: Element, 11: ShadowRoot}
    node = types[nodeType](current_session, result["sharedId"])
    nodeType = current_session.execute_script("""return arguments[0].nodeType""", args=(node,))
    assert nodeType == expected
