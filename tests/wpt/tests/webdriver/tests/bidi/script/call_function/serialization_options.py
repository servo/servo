import pytest
from webdriver.bidi.modules.script import ContextTarget, SerializationOptions
from webdriver.bidi.undefined import UNDEFINED

from ... import any_string, recursive_compare

pytestmark = pytest.mark.asyncio


@pytest.mark.parametrize(
    "include_shadow_tree, shadow_root_mode, contains_children, expected",
    [
        (
            UNDEFINED,
            "open",
            False,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {"nodeType": 11, "childNodeCount": 1},
            },
        ),
        (
            UNDEFINED,
            "closed",
            False,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {"nodeType": 11, "childNodeCount": 1},
            },
        ),
        (
            "none",
            "open",
            False,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {"nodeType": 11, "childNodeCount": 1},
            },
        ),
        (
            "none",
            "closed",
            False,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {"nodeType": 11, "childNodeCount": 1},
            },
        ),
        (
            "open",
            "open",
            True,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "nodeType": 11,
                    "childNodeCount": 1,
                    "children": [
                        {
                            "type": "node",
                            "sharedId": any_string,
                            "value": {
                                "nodeType": 1,
                                "localName": "div",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "childNodeCount": 1,
                                "attributes": {"id": "in-shadow-dom"},
                                "shadowRoot": None,
                            },
                        }
                    ],
                    "mode": "open",
                },
            },
        ),
        (
            "open",
            "closed",
            False,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {"nodeType": 11, "childNodeCount": 1},
            },
        ),
        (
            "all",
            "open",
            True,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "nodeType": 11,
                    "childNodeCount": 1,
                    "children": [
                        {
                            "type": "node",
                            "sharedId": any_string,
                            "value": {
                                "nodeType": 1,
                                "localName": "div",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "childNodeCount": 1,
                                "attributes": {"id": "in-shadow-dom"},
                                "shadowRoot": None,
                            },
                        }
                    ],
                    "mode": "open",
                },
            },
        ),
        (
            "all",
            "closed",
            True,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "nodeType": 11,
                    "childNodeCount": 1,
                    "children": [
                        {
                            "type": "node",
                            "sharedId": any_string,
                            "value": {
                                "nodeType": 1,
                                "localName": "div",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "childNodeCount": 1,
                                "attributes": {"id": "in-shadow-dom"},
                                "shadowRoot": None,
                            },
                        }
                    ],
                    "mode": "closed",
                },
            },
        ),
    ],
    ids=[
        "default mode for open shadow root",
        "default mode for closed shadow root",
        "'none' mode for open shadow root",
        "'none' mode for closed shadow root",
        "'open' mode for open shadow root",
        "'open' mode for closed shadow root",
        "'all' mode for open shadow root",
        "'all' mode for closed shadow root",
    ],
)
async def test_include_shadow_tree_for_custom_element(
    bidi_session,
    top_context,
    get_test_page,
    include_shadow_tree,
    shadow_root_mode,
    contains_children,
    expected,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(shadow_root_mode=shadow_root_mode),
        wait="complete",
    )
    result = await bidi_session.script.call_function(
        function_declaration="""() => document.querySelector("custom-element")""",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        serialization_options=SerializationOptions(
            include_shadow_tree=include_shadow_tree, max_dom_depth=1
        ),
    )

    recursive_compare(expected, result["value"]["shadowRoot"])

    # Explicitely check for children because recursive_compare skips it
    if not contains_children:
        assert "children" not in result["value"]["shadowRoot"]["value"]


@pytest.mark.parametrize(
    "include_shadow_tree, contains_children, expected",
    [
        (
            UNDEFINED,
            False,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {"childNodeCount": 1, "mode": "open", "nodeType": 11},
            },
        ),
        (
            "none",
            False,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {"childNodeCount": 1, "mode": "open", "nodeType": 11},
            },
        ),
        (
            "open",
            True,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 1,
                    "children": [
                        {
                            "type": "node",
                            "sharedId": any_string,
                            "value": {
                                "nodeType": 1,
                                "localName": "div",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "childNodeCount": 1,
                                "attributes": {"id": "in-shadow-dom"},
                                "shadowRoot": None,
                            },
                        }
                    ],
                    "nodeType": 11,
                    "mode": "open",
                },
            },
        ),
        (
            "all",
            True,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 1,
                    "children": [
                        {
                            "type": "node",
                            "sharedId": any_string,
                            "value": {
                                "nodeType": 1,
                                "localName": "div",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "childNodeCount": 1,
                                "attributes": {"id": "in-shadow-dom"},
                                "shadowRoot": None,
                            },
                        }
                    ],
                    "mode": "open",
                    "nodeType": 11,
                },
            },
        ),
    ],
    ids=[
        "default mode",
        "'none' mode",
        "'open' mode",
        "'all' mode",
    ],
)
async def test_include_shadow_tree_for_shadow_root(
    bidi_session,
    top_context,
    get_test_page,
    include_shadow_tree,
    contains_children,
    expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(),
        wait="complete",
    )
    result = await bidi_session.script.call_function(
        function_declaration="""() => document.querySelector("custom-element").shadowRoot""",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        serialization_options=SerializationOptions(
            include_shadow_tree=include_shadow_tree, max_dom_depth=1
        ),
    )

    recursive_compare(expected, result)

    # Explicitely check for children because recursive_compare skips it
    if not contains_children:
        assert "children" not in result["value"]


@pytest.mark.parametrize(
    "max_dom_depth, expected",
    [
        (
            None,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {"id": "with-children"},
                    "childNodeCount": 2,
                    "localName": "div",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                    "shadowRoot": None,
                },
            },
        ),
        (
            0,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {"id": "with-children"},
                    "childNodeCount": 2,
                    "localName": "div",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                    "shadowRoot": None,
                },
            },
        ),
        (
            1,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {"id": "with-children"},
                    "childNodeCount": 2,
                    "children": [
                        {
                            "sharedId": any_string,
                            "type": "node",
                            "value": {
                                "attributes": {},
                                "childNodeCount": 1,
                                "localName": "p",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "nodeType": 1,
                                "shadowRoot": None,
                            },
                        },
                        {
                            "sharedId": any_string,
                            "type": "node",
                            "value": {
                                "attributes": {},
                                "childNodeCount": 0,
                                "localName": "br",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "nodeType": 1,
                                "shadowRoot": None,
                            },
                        },
                    ],
                    "localName": "div",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                    "shadowRoot": None,
                },
            },
        ),
        (
            2,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {"id": "with-children"},
                    "childNodeCount": 2,
                    "children": [
                        {
                            "sharedId": any_string,
                            "type": "node",
                            "value": {
                                "attributes": {},
                                "childNodeCount": 1,
                                "children": [
                                    {
                                        "type": "node",
                                        "sharedId": any_string,
                                        "value": {
                                            "nodeType": 1,
                                            "localName": "span",
                                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                                            "childNodeCount": 0,
                                            "attributes": {},
                                            "shadowRoot": None,
                                        },
                                    }
                                ],
                                "localName": "p",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "nodeType": 1,
                                "shadowRoot": None,
                            },
                        },
                        {
                            "sharedId": any_string,
                            "type": "node",
                            "value": {
                                "attributes": {},
                                "childNodeCount": 0,
                                "localName": "br",
                                "namespaceURI": "http://www.w3.org/1999/xhtml",
                                "nodeType": 1,
                                "shadowRoot": None,
                            },
                        },
                    ],
                    "localName": "div",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                    "shadowRoot": None,
                },
            },
        ),
    ],
)
async def test_max_dom_depth(
    bidi_session, top_context, get_test_page, max_dom_depth, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=get_test_page(), wait="complete"
    )
    result = await bidi_session.script.call_function(
        function_declaration="""() => document.querySelector("div#with-children")""",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        serialization_options=SerializationOptions(
            max_dom_depth=max_dom_depth),
    )

    recursive_compare(expected, result)


async def test_max_dom_depth_null(
    bidi_session,
    top_context,
    get_test_page,
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=get_test_page(), wait="complete"
    )
    result = await bidi_session.script.call_function(
        function_declaration="""() => document.querySelector("div#with-children")""",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        serialization_options=SerializationOptions(max_dom_depth=None),
    )

    recursive_compare(
        {
            "type": "node",
            "sharedId": any_string,
            "value": {
                "nodeType": 1,
                "localName": "div",
                "namespaceURI": "http://www.w3.org/1999/xhtml",
                "childNodeCount": 2,
                "children": [
                    {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "nodeType": 1,
                            "localName": "p",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "childNodeCount": 1,
                            "children": [
                                {
                                    "type": "node",
                                    "sharedId": any_string,
                                    "value": {
                                        "nodeType": 1,
                                        "localName": "span",
                                        "namespaceURI": "http://www.w3.org/1999/xhtml",
                                        "childNodeCount": 0,
                                        "children": [],
                                        "attributes": {},
                                        "shadowRoot": None,
                                    },
                                }
                            ],
                            "attributes": {},
                            "shadowRoot": None,
                        },
                    },
                    {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "nodeType": 1,
                            "localName": "br",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "childNodeCount": 0,
                            "children": [],
                            "attributes": {},
                            "shadowRoot": None,
                        },
                    },
                ],
                "attributes": {"id": "with-children"},
                "shadowRoot": None,
            },
        },
        result,
    )


@pytest.mark.parametrize(
    "max_object_depth, expected",
    [
        (
            UNDEFINED,
            {
                "type": "array",
                "value": [
                    {"type": "number", "value": 1},
                    {"type": "array", "value": [{"type": "number", "value": 2}]},
                ],
            },
        ),
        (0, {"type": "array"}),
        (
            1,
            {
                "type": "array",
                "value": [
                    {"type": "number", "value": 1},
                    {"type": "array"},
                ],
            },
        ),
        (
            2,
            {
                "type": "array",
                "value": [
                    {"type": "number", "value": 1},
                    {"type": "array", "value": [{"type": "number", "value": 2}]},
                ],
            },
        ),
    ],
)
async def test_max_object_depth(bidi_session, top_context, max_object_depth, expected):
    result = await bidi_session.script.call_function(
        function_declaration="() => [1, [2]]",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
        serialization_options=SerializationOptions(max_object_depth=max_object_depth),
    )

    assert result == expected
