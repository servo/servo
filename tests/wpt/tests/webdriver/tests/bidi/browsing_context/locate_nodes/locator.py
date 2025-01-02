import pytest

from ... import any_string, recursive_compare


@pytest.mark.parametrize("type,value", [
    ("css", "div"),
    ("xpath", "//div"),
    ("innerText", "foobarBARbaz"),
    ("accessibility", {"role": "banner"}),
    ("accessibility", {"name": "foo"}),
    ("accessibility", {"role": "banner", "name": "foo"}),
])
@pytest.mark.asyncio
async def test_find_by_locator(bidi_session, inline, top_context, type, value):
    url = inline("""
        <div data-class="one" role="banner" aria-label="foo">foobarBARbaz</div>
        <div data-class="two" role="banner" aria-label="foo">foobarBARbaz</div>
    """)
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
     }, ["html"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "full",
         "maxDepth": 0,
         "value": "foobarBARbaz"
     }, ["html"]),
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "partial",
         "maxDepth": 0,
         "value": "bar"
     }, ["html"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "partial",
         "maxDepth": 0,
         "value": "BAR"
     }, ["html"]),
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "full",
         "maxDepth": 2,
         "value": "foobarbarbaz"
     }, ["div"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "full",
         "maxDepth": 2,
         "value": "foobarBARbaz"
     }, ["div"]),
    ({
         "type": "innerText",
         "ignoreCase": True,
         "matchType": "partial",
         "maxDepth": 2,
         "value": "bar"
     }, ["div"]),
    ({
         "type": "innerText",
         "ignoreCase": False,
         "matchType": "partial",
         "maxDepth": 2,
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
    "ignore_case_true_full_match_max_depth_two",
    "ignore_case_false_full_match_max_depth_two",
    "ignore_case_true_partial_match_max_depth_two",
    "ignore_case_false_partial_match_max_depth_two",
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


@pytest.mark.parametrize(
    "html,locator_value,expected_node_local_name",
    [
        (
            "<article data-class='one'>foo</article><div data-class='two'>bar</div>",
            {"role": "article"},
            "article",
        ),
        (
            "<input role='searchbox' data-class='one' /><input data-class='two' type='text'/>",
            {"role": "searchbox"},
            "input",
        ),
        (
            "<button data-class='one'>Ok</button><button data-class='two'>Cancel</button>",
            {"name": "Ok"},
            "button",
        ),
        (
            "<button data-class='one' aria-labelledby='one two'></button><div id='one'>ok</div><div id='two'>go</div><button data-class='two'>Cancel</button>",
            {"name": "ok go"},
            "button",
        ),
        (
            "<button data-class='one' aria-label='foo'>bar</button><button data-class='two' aria-label='bar'>foo</button>",
            {"name": "foo"},
            "button",
        ),
        (
            "<div role='banner' aria-label='foo' data-class='one'></div><div role='banner'  data-class='two'></div><div aria-label='foo' data-class='three'></div>",
            {"role": "banner", "name": "foo"},
            "div",
        ),
    ],
)
@pytest.mark.asyncio
async def test_locate_by_accessibility_attributes(
    bidi_session,
    inline,
    top_context,
    html,
    locator_value,
    expected_node_local_name,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=inline(html), wait="complete"
    )

    expected = [
        {
            "type": "node",
            "value": {
                "attributes": {"data-class": "one"},
                "localName": expected_node_local_name,
            },
        }
    ]

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={"type": "accessibility", "value": locator_value},
    )

    recursive_compare(expected, result["nodes"])
