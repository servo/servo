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


@pytest.mark.parametrize("locator,expected_nodes_values", [
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "full",
         "value": "bar"
     }, ["strong", "span"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "full",
         "value": "BAR"
     }, ["span"]),
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "partial",
         "value": "ba"
     }, ["strong", "span"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "partial",
         "value": "ba"
     }, ["strong"]),
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "full",
         "maxDepth": 0,
         "value": "foobarbarbaz"
     }, ["body"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "full",
         "maxDepth": 0,
         "value": "foobarBARbaz"
     }, ["body"]),
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "partial",
         "maxDepth": 0,
         "value": "bar"
     }, ["body"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "partial",
         "maxDepth": 0,
         "value": "BAR"
     }, ["body"]),
    ({

         "type": "innerText",
         "ignoreCase": True,
         "matchType": "full",
         "maxDepth": 1,
         "value": "foobarbarbaz"
     }, ["div"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "full",
         "maxDepth": 1,
         "value": "foobarBARbaz"
     }, ["div"]),
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "partial",
         "maxDepth": 1,
         "value": "bar"
     }, ["div"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "partial",
         "maxDepth": 1,
         "value": "BAR"
     }, ["div"]),
], ids=[
    "ignore_case_true_full_match_no_max_depth",
    "ignore_case_false_full_match_no_max_depth",
    "ignore_case_true_partial_match_no_max_depth",
    "ignore_case_false_partial_match_no_max_depth",
    "ignore_case_true_full_match_max_depth_zero",
    "ignore_case_false_full_match_max_depth_zero",
    "ignore_case_true_partial_match_max_depth_zero",
    "ignore_case_false_partial_match_max_depth_zero",
    "ignore_case_true_full_match_max_depth_one",
    "ignore_case_false_full_match_max_depth_one",
    "ignore_case_true_partial_match_max_depth_one",
    "ignore_case_false_partial_match_max_depth_one",
])
@pytest.mark.asyncio
async def test_find_by_inner_text(bidi_session, inline, top_context, locator, expected_nodes_values):
    url = inline("""<div>foo<span><strong>bar</strong></span><span>BAR</span>baz</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    # Construct expected nodes list with the expected nodes values emitting other fields.
    expected = [{
        "type": "node",
        "value": {
            "localName": node_value,
        }
    } for node_value in expected_nodes_values]

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator=locator
    )

    recursive_compare(expected, result["nodes"])
