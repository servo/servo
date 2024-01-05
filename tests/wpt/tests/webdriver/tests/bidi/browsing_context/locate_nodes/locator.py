import pytest

from ... import any_string, recursive_compare


@pytest.mark.parametrize("type,value", [
    ("css", "div"),
    ("xpath", "//div"),
    ("innerText", "foobarBARbaz")
])
@pytest.mark.asyncio
async def test_find_by_locator(bidi_session, inline, top_context, type, value):
    url = inline("""<div data-class="one">foobarBARbaz</div><div data-class="two">foobarBARbaz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": type, "value": value }
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


@pytest.mark.parametrize("ignore_case,match_type,max_depth,value,expected", [
    (True, "full", None, "bar", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 1,
                "children": [],
                "localName": "strong",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        },
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 1,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    ),
    (False, "full", None, "BAR", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 1,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    ),
    (True, "partial", None, "ba", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 1,
                "localName": "strong",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        },
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 1,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    ),
    (False, "partial", None, "ba", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 1,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    ),
    (True, "full", 0, "foobarbarbaz", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 4,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    ),
    (False, "full", 0, "foobarBARbaz", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 4,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    ),
    (True, "partial", 0, "bar", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 4,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    ),
    (False, "partial", 0, "BAR", [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {},
                "childNodeCount": 4,
                "localName": "span",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }]
    )
], ids=[
    "ignore_case_true_full_match_no_max_depth",
    "ignore_case_false_full_match_no_max_depth",
    "ignore_case_true_partial_match_no_max_depth",
    "ignore_case_false_partial_match_no_max_depth",
    "ignore_case_true_full_match_max_depth_zero",
    "ignore_case_false_full_match_max_depth_zero",
    "ignore_case_true_partial_match_max_depth_zero",
    "ignore_case_false_partial_match_max_depth_zero",
])
@pytest.mark.asyncio
async def test_find_by_inner_text(bidi_session, inline, top_context, ignore_case, match_type, max_depth, value, expected):
    url = inline("""<div>foo<span><strong>bar</strong></span><span>BAR</span>baz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={
            "type": "innerText",
            "value": value,
            "ignoreCase": ignore_case,
            "matchType": match_type,
            "maxDepth": max_depth
        }
    )

    recursive_compare(expected, result["nodes"])
