import pytest

from webdriver.bidi.modules.script import ContextTarget
from ... import any_string, recursive_compare


@pytest.mark.parametrize("type,value", [
    ("css", "div"),
    ("xpath", "//div"),
    ("innerText", "foo")
])
@pytest.mark.asyncio
async def test_locate_with_context_nodes(bidi_session, inline, top_context, type, value):
    url = inline("""<p id="parent"><div data-class="one">foo</div><div data-class="two">foo</div></p>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    context_nodes = await bidi_session.script.evaluate(
        expression="""document.querySelector("p")""",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": type, "value": value },
        start_nodes=[context_nodes]
    )

    expected = [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        },
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"two"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }
    ]

    recursive_compare(expected, result["nodes"])


@pytest.mark.parametrize("type,value", [
    ("css", "div[data-class='one']"),
    ("xpath", ".//div[@data-class='one']"),
    ("innerText", "foo")
])
@pytest.mark.asyncio
async def test_locate_with_multiple_context_nodes(bidi_session, inline, top_context, type, value):
    url = inline("""
                 <p id="parent-one"><div data-class="one">foo</div><div data-class="two">bar</div></p>
                 <p id="parent-two"><div data-class="one">foo</div><div data-class="two">bar</div></p>
                 """)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    script_result = await bidi_session.script.evaluate(
        expression="""document.querySelectorAll("p")""",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )

    context_nodes = script_result["value"]

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": type, "value": value },
        start_nodes=context_nodes
    )

    expected = [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        },
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }
    ]

    recursive_compare(expected, result["nodes"])
