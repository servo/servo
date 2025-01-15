import pytest
import webdriver.bidi.error as error

from webdriver.bidi.modules.script import ContextTarget

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


@pytest.mark.parametrize("type", ["css", "xpath", "innerText"])
@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_locator_value_invalid_type(
    bidi_session, inline, top_context, type, value
):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"], locator={"type": type, "value": value}
        )


@pytest.mark.parametrize("value", [None, False, 42, {}, []])
async def test_params_locator_accessability_value_invalid_type(
    bidi_session, inline, top_context, value
):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"], locator={"type": "accessability", "value": value}
        )


@pytest.mark.parametrize("type,value", [
    ("css", "a*b"),
    ("xpath", ""),
    ("innerText", ""),
    ("accessibility", {}),
    ("context", {})
])
async def test_params_locator_value_invalid_value(bidi_session, inline, top_context, type, value):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.InvalidSelectorException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"], locator={ "type": type, "value": value }
        )


async def test_params_locator_xpath_unknown_error(bidi_session, inline, top_context):
    await navigate_to_page(bidi_session, inline, top_context)

    with pytest.raises(error.UnknownErrorException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"], locator={"type": "xpath", "value": "/foo:bar"}
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
            locator={ "type": "css", "value": "div" },
            start_nodes=[]
        )


@pytest.mark.parametrize(
    "value",
    [
        {"type": "number", "value": 3},
        {"type": "window"},
        {"type": "array", "value": ["test"]},
        {
            "type": "object",
            "value": [
                ["1", {"type": "string", "value": "foo"}],
            ],
        },
    ],
)
async def test_params_start_nodes_not_dom_node(
    bidi_session, inline, top_context, value
):
    await navigate_to_page(bidi_session, inline, top_context)

    if value["type"] == "window":
        value["value"] = top_context["context"]

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={"type": "css", "value": "div"},
            start_nodes=[value],
        )


@pytest.mark.parametrize(
    "expression",
    [
        "document.querySelector('input#button').attributes[0]",
        "document.querySelector('#with-text-node').childNodes[0]",
        """document.createProcessingInstruction("xml-stylesheet", "href='foo.css'")""",
        "document.querySelector('#with-comment').childNodes[0]",
        "document.doctype",
        "document.getElementsByTagName('div')",
        "document.querySelectorAll('div')"
    ],
)
async def test_params_start_nodes_dom_node_not_element(
    bidi_session, inline, top_context, get_test_page, expression
):
    await navigate_to_page(bidi_session, inline, top_context)

    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    remote_reference = await bidi_session.script.evaluate(
        expression=expression,
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    with pytest.raises(error.InvalidArgumentException):
        await bidi_session.browsing_context.locate_nodes(
            context=top_context["context"],
            locator={"type": "css", "value": "div"},
            start_nodes=[remote_reference],
        )
