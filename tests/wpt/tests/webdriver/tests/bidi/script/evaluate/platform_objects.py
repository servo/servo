import pytest
from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.undefined import UNDEFINED

from ... import any_string, recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "expression",
    [
        "location",
        "new URL('https://example.com')",
        "new Event('foo')",
        "document.all",
        "document.implementation",
    ],
)
async def test_platform_object(bidi_session, top_context, expression):
    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare({"type": "object", "value": UNDEFINED}, result)


async def test_dom_exception(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="new DOMException('foo')",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == {"type": "error"}


async def test_observable_array(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="(() => { document.adoptedStyleSheets = [new CSSStyleSheet()]; return document.adoptedStyleSheets; })()",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare({"type": "array", "value": [{"type": "object", "value": UNDEFINED}]}, result)


async def test_object_with_platform_prototype(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="Object.create(HTMLElement.prototype)",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == {"type": "object", "value": []}


@pytest.mark.parametrize(
    "expression, expected",
    [
        ("document.querySelector('#foo')", {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"id": "foo"},
                "childNodeCount": 0,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
                "shadowRoot": None,
            },
        }),
        ("document.querySelectorAll('div')", {
            "type": "nodelist",
            "value": [{"type": "node", "sharedId": any_string}],
        }),
    ],
)
async def test_page_defined_interface_names(bidi_session, top_context, inline, expression, expected):
    url = inline("""
        <div id="foo"></div>
        <script>
            class Node {}
            class Element {}
            class NodeList {}
        </script>
    """)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)
