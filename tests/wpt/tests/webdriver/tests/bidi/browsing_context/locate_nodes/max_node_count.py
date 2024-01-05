import pytest

from ... import any_string, recursive_compare


@pytest.mark.parametrize("type,value,max_count,expected", [
    ("css", "div", 1, [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            },
        }]
    ),
    ("xpath", "//div", 1, [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            },
        }]
    ),
    ("innerText", "foo", 1, [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            },
        }]
    ),
    ("css", "div", 10, [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            },
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
            },
        }]
    ),
    ("xpath", "//div", 10, [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            },
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
            },
        }]
    ),
    ("innerText", "foo", 10, [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"data-class":"one"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            },
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
            },
        }]
    )
], ids=[
    "css_single",
    "xpath_single",
    "inner_text_single",
    "css_multiple",
    "xpath_multiple",
    "inner_text_multiple"
])
@pytest.mark.asyncio
async def test_find_by_locator_limit_return_count(bidi_session, inline, top_context, type, value, max_count, expected):
    url = inline("""<div data-class="one">foo</div><div data-class="two">foo</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": type, "value": value },
        max_node_count = max_count
    )

    recursive_compare(expected, result["nodes"])


@pytest.mark.asyncio
async def test_several_context_nodes(bidi_session, inline, top_context):
    url = inline(
        """
        <div class="context-node">
            <div>should be returned</div>
        </div>
        <div class="context-node">
            <div>should not be returned</div>
            <div>should not be returned</div>
            <div>should not be returned</div>
            <div>should not be returned</div>
            <div>should not be returned</div>
            <div>should not be returned</div>
            <div>should not be returned</div>
            <div>should not be returned</div>
            <div>should not be returned</div>
        </div>
    """
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result_context_nodes = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": "css", "value": ".context-node"},
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": "css", "value": "div"},
        max_node_count=1,
        start_nodes=[
            {"sharedId": result_context_nodes["nodes"][0]["sharedId"]},
            {"sharedId": result_context_nodes["nodes"][1]["sharedId"]},
        ],
    )

    assert len(result["nodes"]) == 1
