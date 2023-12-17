import pytest
import webdriver.bidi.error as error

pytestmark = pytest.mark.asyncio


MAX_INT = 9007199254740991


async def navigate_to_page(bidi_session, inline, top_context):
    url = inline("""<div>foo</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_context_invalid_type(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=value, locator={"type": "css", "value": "div"}
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_locator_type_invalid_type(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"], locator={ "type": value, "value": "div" }
        )


@pytest.mark.parametrize("type", ["", "invalid"])
async def test_params_locator_type_invalid_value(bidi_session, inline, top_context, type):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"], locator={ "type": type, "value": "div" }
        )


@pytest.mark.parametrize("type,value", [
    ("css", "a*b"),
    ("xpath", ""),
    ("xpath", "invalid-xpath")
    ("innerText", "")
])
async def test_params_locator_value_invalid_value(bidi_session, inline, top_context, type, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidSelectorException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"], locator={ "type": type, "value": value }
        )


@pytest.mark.parametrize("value", [False, "string", 1.5, {}, []])
async def test_params_max_node_count_invalid_type(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "css", "value": "div" },
            max_node_count=value
        )


@pytest.mark.parametrize("value", [0, MAX_INT + 1])
async def test_params_max_node_count_invalid_value(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "invalid", "value": "div" },
            max_node_count=value
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_ownership_invalid_type(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "css", "value": "div" },
            ownership=value
        )


async def test_params_ownership_invalid_value(bidi_session, inline, top_context):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "css", "value": "div" },
            ownership="foo"
        )


@pytest.mark.parametrize("value", [False, 42, {}, []])
async def test_params_sandbox_invalid_type(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "css", "value": "div" },
            sandbox=value
        )


@pytest.mark.parametrize("value", [False, 42, "foo", []])
async def test_params_serialization_options_invalid_type(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "css", "value": "div" },
            serialization_options=value
        )


@pytest.mark.parametrize("value", [False, "string", 42, {}])
async def test_params_start_nodes_invalid_type(bidi_session, inline, top_context, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "css", "value": "div" },
            start_nodes=value
        )


async def test_params_start_nodes_empty_list(bidi_session, inline, top_context):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={ "type": "invalid", "value": "div" },
            start_nodes=[]
        )
