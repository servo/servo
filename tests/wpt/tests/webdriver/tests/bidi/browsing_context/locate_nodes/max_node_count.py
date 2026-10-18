import pytest
from webdriver.bidi.undefined import UNDEFINED

from ... import any_string, recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize("locator_type,value,max_count,expected", [
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
    ("accessibility", {"role": "banner"}, 1, [
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
    ("accessibility", {"name": "bar"}, 1, [
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
    ("accessibility", {"role": "banner", "name": "bar"}, 1, [
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
    ),
    ("accessibility", {"role": "banner"}, 10, [
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
        }, {
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
    ("accessibility", {"name": "bar"}, 10, [
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
        }, {
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
    ("accessibility", {"role": "banner", "name": "bar"}, 10, [
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
        }, {
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
], ids=[
    "css_single",
    "xpath_single",
    "inner_text_single",
    "accessibility_role_single",
    "accessibility_name_single",
    "accessibility_role_name_single",
    "css_multiple",
    "xpath_multiple",
    "inner_text_multiple",
    "accessibility_role_multiple",
    "accessibility_name_multiple",
    "accessibility_role_name_multiple",
])
async def test_find_by_locator_limit_return_count(
    bidi_session,
    inline,
    top_context,
    locator_type,
    value,
    max_count,
    expected,
):
    url = inline("""
        <div data-class="one" role="banner" aria-label="bar">foo</div>
        <div data-class="two" role="banner" aria-label="bar">foo</div>
    """)
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    nodes = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": locator_type, "value": value},
        max_node_count=max_count,
    )

    recursive_compare(expected, nodes)


@pytest.mark.parametrize("locator_type,value", [
    ("css", "div"),
    ("xpath", ".//div"),
    ("innerText", "foo"),
    ("accessibility", {"role": "banner"}),
    ("accessibility", {"name": "bar"}),
    ("accessibility", {"role": "banner", "name": "bar"}),
])
@pytest.mark.parametrize("max_node_count", [1, 3, 4, UNDEFINED])
@pytest.mark.parametrize("reverse_start_nodes", [False, True])
async def test_several_context_nodes(bidi_session, inline, top_context,
                                     locator_type, value, max_node_count,
                                     reverse_start_nodes):
    url = inline(
        """
        <section class="context-node">
            <div id="first" role="banner" aria-label="bar">foo</div>
        </section>
        <section class="context-node">
            <div id="second-a" role="banner" aria-label="bar">foo</div>
            <div id="second-b" role="banner" aria-label="bar">foo</div>
            <div id="second-c" role="banner" aria-label="bar">foo</div>
        </section>
    """
    )
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    context_nodes = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": "css", "value": ".context-node"},
    )
    start_nodes = [
        {"sharedId": node["sharedId"]} for node in context_nodes
    ]
    expected_ids = ["first", "second-a", "second-b", "second-c"]
    if reverse_start_nodes:
        start_nodes.reverse()
        expected_ids = ["second-a", "second-b", "second-c", "first"]

    nodes = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": locator_type, "value": value},
        max_node_count=max_node_count,
        start_nodes=start_nodes,
    )

    if max_node_count is not UNDEFINED:
        expected_ids = expected_ids[:max_node_count]
    assert [node["value"]["attributes"]["id"] for node in nodes] == expected_ids
