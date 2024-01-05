import pytest

from webdriver.bidi.modules.script import SerializationOptions
from ... import any_string, recursive_compare


@pytest.mark.parametrize("mode", [
    "open",
    "closed"
])
@pytest.mark.asyncio
async def test_locate_nodes_serialization_options(bidi_session, top_context, get_test_page, mode):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(shadow_root_mode=mode),
        wait="complete",
    )

    result = await bidi_session.browsing_context.locate_nodes(
        context=top_context["context"],
        locator={ "type": "css", "value": "custom-element" },
        serialization_options=SerializationOptions(include_shadow_tree="all", max_dom_depth=1)
    )

    expected = [
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "attributes": {
                    "id": "custom-element",
                },
                "childNodeCount": 0,
                "localName": "custom-element",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "nodeType": 1,
                "shadowRoot": {
                    "type": "node",
                    "sharedId": any_string,
                    "value": {
                        "childNodeCount": 1,
                        "children": [
                            {
                                "type": "node",
                                "sharedId": any_string,
                                "value": {
                                    "attributes": {
                                        "id": "in-shadow-dom"
                                    },
                                    "childNodeCount": 1,
                                    "localName": "div",
                                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                                    "nodeType": 1
                                }
                            }
                        ],
                        "mode": mode,
                        "nodeType": 11,
                    }
                },
            }
        }
    ]

    recursive_compare(expected, result["nodes"])
