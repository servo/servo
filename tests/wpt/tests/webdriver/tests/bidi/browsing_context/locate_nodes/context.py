import pytest
import webdriver.bidi.error as error

from ... import any_string, recursive_compare


@pytest.mark.asyncio
async def test_params_context_invalid_value(bidi_session, inline, top_context):
    url = inline("""<div>foo</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    with pytest.raises(error.NoSuchFrameException):
        await bidi_session.browsing_context.locate_nodes(
            context="foo", locator={ "type": "css", "value": "div" }
        )


@pytest.mark.asyncio
async def test_locate_in_different_contexts(bidi_session, inline, top_context, new_tab):
    url = inline("""<div class="in-top-context">foo</div>""")
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=url, wait="complete"
    )

    # Try to locate nodes in the other context
    result = await bidi_session.browsing_context.locate_nodes(
        context=new_tab["context"], locator={"type": "css", "value": ".in-top-context"}
    )

    assert result["nodes"] == []

    # Locate in the correct context
    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"], locator={"type": "css", "value": ".in-top-context"}
    )

    expected = [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"class": "in-top-context"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }
    ]

    recursive_compare(expected, result["nodes"])


@pytest.mark.parametrize("domain", ["", "alt"], ids=["same_origin", "cross_origin"])
@pytest.mark.asyncio
async def test_locate_in_iframe(bidi_session, inline, top_context, domain):
    iframe_url_1 = inline("<div id='in-iframe'>foo</div>", domain=domain)
    page_url = inline(f"<iframe src='{iframe_url_1}'></iframe>")

    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=page_url, wait="complete"
    )

    contexts = await bidi_session.browsing_context.get_tree(root=top_context["context"])
    iframe_context = contexts[0]["children"][0]

    result = await bidi_session.browsing_context.locate_nodes(
        context=iframe_context["context"],
        locator={"type": "css", "value": "#in-iframe"}
    )

    expected = [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {"id": "in-iframe"},
                "childNodeCount": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
            }
        }
    ]

    recursive_compare(expected, result["nodes"])
