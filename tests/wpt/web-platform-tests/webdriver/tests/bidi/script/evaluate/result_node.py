import pytest
from webdriver.bidi.modules.script import ContextTarget


page_data = """
    <div id="deep"><p><span></span></p><br/></div>
    <div id="text-node"><p></p>Lorem</div>
    <br/>
    <svg id="foo"></svg>
    <div id="comment"><!-- Comment --></div>
    <script>
        var svg = document.querySelector("svg");
        svg.setAttributeNS("http://www.w3.org/2000/svg", "svg:foo", "bar");
    </script>
"""


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (   # basic
            """
                document.querySelector("br")
            """,
            {
                "type": "node",
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
                document.querySelector("svg")
            """,
            {
                "type": "node",
                "value": {
                    "attributes": {
                        "id": "foo",
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
                document.querySelector("div#text-node")
            """,
            {
                "type": "node",
                "value": {
                    "attributes": {"id": "text-node"},
                    "childNodeCount": 2,
                    "children": [{
                        "type": "node",
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "children": None,
                            "localName": "p",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    },{
                        "type": "node",
                        "value": {
                            "childNodeCount": 0,
                            "children": None,
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
                document.querySelector("div#deep")
            """,
            {
                "type": "node",
                "value": {
                    "attributes": {"id": "deep"},
                    "childNodeCount": 2,
                    "children": [{
                        "type": "node",
                        "value": {
                            "attributes": {},
                            "childNodeCount": 1,
                            "children": None,
                            "localName": "p",
                            "namespaceURI": "http://www.w3.org/1999/xhtml",
                            "nodeType": 1
                        }
                    },{
                        "type": "node",
                        "value": {
                            "attributes": {},
                            "childNodeCount": 0,
                            "children": None,
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
                document.createElement("div")
            """,
            {
                "type": "node",
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
async def test_element_node(bidi_session, inline, top_context, expression, expected):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                document.querySelector("svg").attributes[0]
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
                    "localName": "id",
                    "namespaceURI": None,
                    "nodeType": 2,
                    "nodeValue": "foo",
                },
            },
        ),(
            """
                document.querySelector("svg").attributes[1]
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
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
async def test_attribute_node(bidi_session, inline, top_context, expression, expected):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                document.querySelector("div#text-node").childNodes[1]
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
                    "nodeType": 3,
                    "nodeValue": "Lorem",
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_text_node(bidi_session, inline, top_context, expression, expected):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                document.querySelector("foo").childNodes[1]
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
                    "nodeType": 4,
                    "nodeValue": " < > & ",
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_cdata_node(bidi_session, inline, new_tab, expression, expected):
    xml_page = inline("""<foo>CDATA section: <![CDATA[ < > & ]]>.</foo>""", doctype="xml")

    result = await bidi_session.browsing_context.navigate(
        context=new_tab['context'], url=xml_page, wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                document.createProcessingInstruction("xml-stylesheet", "href='foo.css'")
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
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
    bidi_session, inline, new_tab, expression, expected
):
    xml_page = inline("""<foo></foo>""", doctype="xml")

    result = await bidi_session.browsing_context.navigate(
        context=new_tab['context'], url=xml_page, wait="complete"
    )


    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(new_tab["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                document.querySelector("div#comment").childNodes[0]
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
                    "nodeType": 8,
                    "nodeValue": " Comment ",
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_comment_node(bidi_session, inline, top_context, expression, expected):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                document
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 2,
                    "children": [{
                        "type": "node",
                        "value": {
                            "childNodeCount": 0,
                            "children": None,
                            "nodeType": 10
                        }
                    }, {
                        "type": "node",
                        "value": {
                            "attributes": {},
                            "childNodeCount": 2,
                            "children": None,
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
async def test_document_node(bidi_session, inline, top_context, expression, expected):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                document.doctype
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
                    "nodeType": 10,
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_doctype_node(bidi_session, inline, top_context, expression, expected):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, expected",
    [
        (
            """
                new DocumentFragment();
            """,
            {
                "type": "node",
                "value": {
                    "childNodeCount": 0,
                    "children": [],
                    "nodeType": 11,
                }
            }
        ),
    ], ids=[
        "basic",
    ]
)
async def test_document_fragment_node(
    bidi_session, inline, top_context, expression, expected
):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == expected


@pytest.mark.asyncio
async def test_node_within_object(bidi_session, inline, top_context):
    result = await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=inline(page_data), wait="complete"
    )

    result = await bidi_session.script.evaluate(
        expression="""({"elem": document.querySelector("span")})""",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert result == {
        "type": "object",
        "value": [
            ["elem", {
                "type": "node",
                "value": {
                    "attributes": {},
                    "childNodeCount": 0,
                    "children": None,
                    "localName": "span",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1
                }
            }]
        ]
    }
