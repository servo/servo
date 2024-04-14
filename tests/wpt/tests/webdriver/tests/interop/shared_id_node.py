import pytest

from webdriver import ShadowRoot, WebElement
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio

DOCUMENT_FRAGMENT_NODE = 11
ELEMENT_NODE = 1


async def test_web_element_reference_created_in_classic(
    bidi_session,
    current_session,
    get_test_page,
    top_context,
):
    current_session.url = get_test_page()

    node = current_session.execute_script(
        """return document.querySelector("div#with-children")"""
    )
    shared_id = node.id

    # Use element reference from WebDriver classic in WebDriver BiDi
    result = await bidi_session.script.call_function(
        function_declaration="(node)=>{return node.nodeType}",
        arguments=[{"sharedId": shared_id}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "number", "value": ELEMENT_NODE}


async def test_web_element_reference_created_in_bidi(
    bidi_session,
    current_session,
    get_test_page,
    top_context,
):
    current_session.url = get_test_page()

    result = await bidi_session.script.evaluate(
        expression="""document.querySelector("div#with-children")""",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    nodeType = result["value"]["nodeType"]
    assert nodeType == ELEMENT_NODE

    # Use element reference from WebDriver BiDi in WebDriver classic
    node = WebElement(current_session, result["sharedId"])
    nodeType = current_session.execute_script(
        """return arguments[0].nodeType""", args=(node,)
    )
    assert nodeType == ELEMENT_NODE


@pytest.mark.parametrize("shadow_root_mode", ["open", "closed"])
async def test_shadow_root_reference_created_in_classic(
    bidi_session, current_session, get_test_page, top_context, shadow_root_mode
):
    current_session.url = get_test_page(shadow_root_mode=shadow_root_mode)

    node = current_session.execute_script(
        """return document.querySelector("custom-element")"""
    )
    shared_id = node.shadow_root.id

    # Use shadow root reference from WebDriver classic in WebDriver BiDi
    result = await bidi_session.script.call_function(
        function_declaration="(node)=>{return node.nodeType}",
        arguments=[{"sharedId": shared_id}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "number", "value": DOCUMENT_FRAGMENT_NODE}


@pytest.mark.parametrize("shadow_root_mode", ["open", "closed"])
async def test_shadow_root_reference_created_in_bidi(
    bidi_session, current_session, get_test_page, top_context, shadow_root_mode
):
    current_session.url = get_test_page(shadow_root_mode=shadow_root_mode)

    result = await bidi_session.script.evaluate(
        expression="""document.querySelector("custom-element")""",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )
    shared_id_for_shadow_root = result["value"]["shadowRoot"]["sharedId"]

    # Use shadow root reference from WebDriver BiDi in WebDriver classic
    node = ShadowRoot(current_session, shared_id_for_shadow_root)
    nodeType = current_session.execute_script(
        """return arguments[0].nodeType""", args=(node,)
    )
    assert nodeType == DOCUMENT_FRAGMENT_NODE
