import pytest

from webdriver.bidi.modules.script import ContextTarget
from ... import any_string, recursive_compare


@pytest.mark.parametrize("type,value,expected", [
    ("css", "p", [{
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "p",
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
                "localName": "p",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
    }]),
    ("css", "a span", [{
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"id":"text"},
                "childNodeCount": 1,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
    }]),
    ("css", "#text", [{
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"id":"text"},
                "childNodeCount": 1,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
    }]),
    ("xpath", "//p", [{
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "p",
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
                "localName": "p",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
    }]),
    ("innerText", "foo", [{
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "p",
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
                "localName": "p",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
    }])
])
@pytest.mark.asyncio
async def test_locate_with_context_nodes(bidi_session, inline, top_context, type, value, expected):
    url = inline("""<div id="parent">
        <p data-class="one">foo</p>
        <p data-class="two">foo</p>
        <a data-class="three">
            <span id="text">bar</span>
        </a>
    </div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    context_nodes = await bidi_session.script.evaluate(
        expression="""document.querySelector("div")""",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": type, "value": value },
        start_nodes=[context_nodes]
    )

    recursive_compare(expected, result["nodes"])


@pytest.mark.parametrize("type,value", [
    ("css", "p[data-class='one']"),
    ("xpath", ".//p[@data-class='one']"),
    ("innerText", "foo")
])
@pytest.mark.asyncio
async def test_locate_with_multiple_context_nodes(bidi_session, inline, top_context, type, value):
    url = inline("""
                 <div id="parent-one"><p data-class="one">foo</p><p data-class="two">bar</p></div>
                 <div id="parent-two"><p data-class="one">foo</p><p data-class="two">bar</p></div>
                 """)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    script_result = await bidi_session.script.evaluate(
        expression="""document.querySelectorAll("div")""",
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
                "localName": "p",
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
                "localName": "p",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }
    ]

    recursive_compare(expected, result["nodes"])
