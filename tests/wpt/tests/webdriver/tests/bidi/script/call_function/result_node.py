import pytest
from webdriver.bidi.modules.script import ContextTarget, SerializationOptions

from ... import any_string, recursive_compare


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (   # basic
            """
                () => document.querySelector("br")
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {},
                    "childNodeCount": 0,
                    "children": [],
                    "localName": "br",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                },
            },
        ),
        (   # attributes
            """
                () => document.querySelector("svg")
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {
                        "svg:foo": "bar",
                    },
                    "childNodeCount": 0,
                    "children": [],
                    "localName": "svg",
                    "namespaceURI": "http://www.w3.org/2000/svg",
                    "nodeType": 1,
                },
            },
        ),
        (   # all children including non-element nodes
            """
                () => document.querySelector("#with-text-node")
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {"id": "with-text-node"},
                    "childNodeCount": 1,
                    "children": [{
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "childNodeCount": 0,
                            "nodeType": 3,
                            "nodeValue": "Lorem",
                        }
                    }],
                    "localName": "div",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                },
            },
        ),
        (   # children limited due to max depth
            """
                () => document.querySelector("#with-children")
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {"id": "with-children"},
                    "childNodeCount": 2,
                    "children": [{
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 1,
                            "localName": "p",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    }, {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "br",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    }],
                    "localName": "div",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                },
            },
        ),
        (   # not connected
            """
                () => document.createElement("div")
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {},
                    "childNodeCount": 0,
                    "children": [],
                    "localName": "div",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1,
                },
            },
        ),
    ], ids=[
        "basic",
        "attributes",
        "all_children",
        "children_max_depth",
        "not_connected",
    ]
)
async def test_element_node(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        serialization_options=SerializationOptions(max_dom_depth=1),
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document.querySelector("input#button").attributes[0]
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "localName": "id",
                    "namespaceURI": None,
                    "nodeType": 2,
                    "nodeValue": "button",
                },
            },
        ), (
            """
                () => document.querySelector("svg").attributes[0]
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "localName": "foo",
                    "namespaceURI": "http://www.w3.org/2000/svg",
                    "nodeType": 2,
                    "nodeValue": "bar",
                },
            },
        ),
    ], ids=[
        "basic",
        "namespace",
    ]
)
async def test_attribute_node(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document.querySelector("#with-text-node").childNodes[0]
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "nodeType": 3,
                    "nodeValue": "Lorem",
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_text_node(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document.querySelector("foo").childNodes[1]
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "nodeType": 4,
                    "nodeValue": " < > & ",
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_cdata_node(bidi_session, inline, new_tab, function_declaration, expected):
    xml_page = inline("""<foo>CDATA section: <![CDATA[ < > & ]]>.</foo>""", doctype="xml")

    await bidi_session.browsing_context.navigate(
        context=new_tab['context'], url=xml_page, wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document.createProcessingInstruction("xml-stylesheet", "href='foo.css'")
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "nodeType": 7,
                    "nodeValue": "href='foo.css'",
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_processing_instruction_node(
    bidi_session, inline, new_tab, function_declaration, expected
):
    xml_page = inline("""<foo></foo>""", doctype="xml")

    await bidi_session.browsing_context.navigate(
        context=new_tab['context'], url=xml_page, wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document.querySelector("#with-comment").childNodes[0]
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "nodeType": 8,
                    "nodeValue": " Comment ",
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_comment_node(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 2,
                    "children": [{
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "childNodeCount": 0,
                            "nodeType": 10
                        }
                    }, {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 2,
                            "localName": "html",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    }],
                    "nodeType": 9
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_document_node(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        serialization_options=SerializationOptions(max_dom_depth=1),
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document.doctype
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "nodeType": 10,
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_doctype_node(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => document.querySelector("#custom-element").shadowRoot
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 1,
                    "mode": "open",
                    "nodeType": 11
                }
            }
        ),
        (
            """
                () => document.createDocumentFragment()
            """,
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "childNodeCount": 0,
                    "children": [],
                    "nodeType": 11,
                }
            }
        ),
    ], ids=[
        "shadow root",
        "not connected",
    ]
)
async def test_document_fragment_node(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        serialization_options=SerializationOptions(max_dom_depth=1),
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            """
                () => [document.querySelector("img")]
            """,
            {
                "type": "array",
                "value": [
                    {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "img",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1,
                        },
                    },
                ],
            },
        ),
        (
            """
                () => {
                    const map = new Map();
                    map.set(document.querySelector("img"), "elem");
                    return map;
                }
            """,
            {
                "type": "map",
                "value": [[
                    {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "img",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    },
                    {
                        "type": "string",
                        "value": "elem"
                    }
                ]]
            }
        ),
        (
            """
                () => {
                    const map = new Map();
                    map.set("elem", document.querySelector("img"));
                    return map;
                }
            """,
            {
                "type": "map",
                "value": [[
                    "elem", {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "img",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    }
                ]]
            }
        ),
        (
            """
                () => ({"elem": document.querySelector("img")})
            """,
            {
                "type": "object",
                "value": [
                    ["elem", {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "img",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    }]
                ]
            }
        ),
        (
            """
                () => {
                    const set = new Set();
                    set.add(document.querySelector("img"));
                    return set;
                }
            """,
            {
                "type": "set",
                "value": [
                    {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "img",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1,
                        },
                    },
                ],
            },
        ),
    ], ids=[
        "array", "map-key", "map-value", "object", "set"
    ]
)
async def test_node_embedded_within(
    bidi_session, get_test_page, top_context, function_declaration, expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "function_declaration, expected",
    [
        (
            "() => document.getElementsByTagName('img')",
            {
                "type": "htmlcollection",
                "value": [
                    {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "img",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    },
                ]
            }
        ),
        (
            "() => document.querySelectorAll('img')",
            {
                "type": "nodelist",
                "value": [
                    {
                        "type": "node",
                        "sharedId": any_string,
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "localName": "img",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    },
                ]
            }
        ),
    ], ids=[
        "htmlcollection",
        "nodelist"
    ]
)
async def test_node_within_dom_collection(
    bidi_session,
    get_test_page,
    top_context,
    function_declaration,
    expected
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
        serialization_options=SerializationOptions(max_dom_depth=1),
    )

    recursive_compare(expected, result)


@pytest.mark.parametrize("shadow_root_mode", ["open", "closed"])
@pytest.mark.asyncio
async def test_custom_element_with_shadow_root(
    bidi_session, get_test_page, top_context, shadow_root_mode
):
    await bidi_session.browsing_context.navigate(
        context=top_context["context"],
        url=get_test_page(shadow_root_mode=shadow_root_mode),
        wait="complete",
    )

    result = await bidi_session.script.call_function(
        function_declaration="""() => document.querySelector("#custom-element")""",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare({
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
                "sharedId": any_string,
                "type": "node",
                "value": {
                    "childNodeCount": 1,
                    "mode": shadow_root_mode,
                    "nodeType": 11,
                }
            },
        }
    }, result)
