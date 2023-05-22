import pytest
from tests.support.sync import AsyncPoll
import webdriver.bidi.error as error
from webdriver.bidi.modules.script import ContextTarget, SerializationOptions

from ... import any_string, recursive_compare


@pytest.mark.asyncio
async def test_default_arguments(bidi_session, top_context):
    result = await bidi_session.script.call_function(
        function_declaration="(...args) => args",
        await_promise=False,
        target=ContextTarget(top_context["context"]))

    recursive_compare({
        "type": "array",
        "value": []
    }, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "argument, expected",
    [
        ({"type": "undefined"}, "undefined"),
        ({"type": "null"}, "null"),
        ({"type": "string", "value": "foobar"}, "'foobar'"),
        ({"type": "string", "value": "2"}, "'2'"),
        ({"type": "number", "value": "-0"}, "-0"),
        ({"type": "number", "value": "Infinity"}, "Infinity"),
        ({"type": "number", "value": "-Infinity"}, "-Infinity"),
        ({"type": "number", "value": 3}, "3"),
        ({"type": "number", "value": 1.4}, "1.4"),
        ({"type": "boolean", "value": True}, "true"),
        ({"type": "boolean", "value": False}, "false"),
        ({"type": "bigint", "value": "42"}, "42n"),
    ],
)
async def test_primitive_value(bidi_session, top_context, argument, expected):
    result = await bidi_session.script.call_function(
        function_declaration=f"""(arg) => {{
            if (arg !== {expected}) {{
                throw new Error(`Argument should be {expected}, but was ` + arg);
            }}
            return arg;
        }}""",
        arguments=[argument],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(argument, result)


@pytest.mark.asyncio
async def test_primitive_value_NaN(bidi_session, top_context):
    nan_remote_value = {"type": "number", "value": "NaN"}
    result = await bidi_session.script.call_function(
        function_declaration="""(arg) => {
            if (!isNaN(arg)) {
                throw new Error("Argument should be 'NaN', but was " + arg);
            }
            return arg;
        }""",
        arguments=[nan_remote_value],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(nan_remote_value, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "argument, expected_type",
    [
        ({
             "type": "array",
             "value": [
                 {"type": "string", "value": "foobar"},
             ],
         },
         "Array"
         ),
        ({"type": "date", "value": "2022-05-31T13:47:29.000Z"},
         "Date"
         ),
        ({
             "type": "map",
             "value": [
                 ["foobar", {"type": "string", "value": "foobar"}],
             ],
         },
         "Map"
         ),
        ({
             "type": "object",
             "value": [
                 ["foobar", {"type": "string", "value": "foobar"}],
             ],
         },
         "Object"
         ),
        ({"type": "regexp", "value": {"pattern": "foo", "flags": "g"}},
         "RegExp"
         ),
        ({
             "type": "set",
             "value": [
                 {"type": "string", "value": "foobar"},
             ],
         },
         "Set"
         )
    ],
)
async def test_local_value(bidi_session, top_context, argument, expected_type):
    result = await bidi_session.script.call_function(
        function_declaration=f"""(arg) => {{
            if (!(arg instanceof {expected_type})) {{
                const type = Object.prototype.toString.call(arg);
                throw new Error(
                    "Argument type should be {expected_type}, but was " + type
                );
            }}
            return arg;
        }}""",
        arguments=[argument],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    recursive_compare(argument, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "setup_expression, function_declaration, expected",
    [
        (
            "Symbol('foo')",
            "(symbol) => symbol.toString()",
            {"type": "string", "value": "Symbol(foo)"},
        ),
        ("[1,2]", "(array) => array[0]", {"type": "number", "value": 1}),
        (
            "new RegExp('foo')",
            "(regexp) => regexp.source",
            {"type": "string", "value": "foo"},
        ),
        (
            "new Date(1654004849000)",
            "(date) => date.toISOString()",
            {"type": "string", "value": "2022-05-31T13:47:29.000Z"},
        ),
        (
            "new Map([['foo', 'bar']])",
            "(map) => map.get('foo')",
            {"type": "string", "value": "bar"},
        ),
        (
            "new Set(['foo'])",
            "(set) => set.has('foo')",
            {"type": "boolean", "value": True},
        ),
        (
            "{const weakMap = new WeakMap(); weakMap.set(weakMap, 'foo')}",
            "(weakMap)=> weakMap.get(weakMap)",
            {"type": "string", "value": "foo"},
        ),
        (
            "{const weakSet = new WeakSet(); weakSet.add(weakSet)}",
            "(weakSet)=> weakSet.has(weakSet)",
            {"type": "boolean", "value": True},
        ),
        (
            "new Error('error message')",
            "(error) => error.message",
            {"type": "string", "value": "error message"},
        ),
        (
            "new SyntaxError('syntax error message')",
            "(error) => error.message",
            {"type": "string", "value": "syntax error message"},
        ),
        (
            "new Promise((resolve) => resolve(3))",
            "(promise) => promise",
            {"type": "number", "value": 3},
        ),
        (
            "new Int8Array(2)",
            "(int8Array) => int8Array.length",
            {"type": "number", "value": 2},
        ),
        (
            "new ArrayBuffer(8)",
            "(arrayBuffer) => arrayBuffer.byteLength",
            {"type": "number", "value": 8},
        ),
        ("() => true", "(func) => func()", {"type": "boolean", "value": True}),
        (
            "(function() {return false;})",
            "(func) => func()",
            {"type": "boolean", "value": False},
        ),
        (
            "window.foo = 3; window",
            "(window) => window.foo",
            {"type": "number", "value": 3},
        ),
        (
            "window.url = new URL('https://example.com'); window.url",
            "(url) => url.hostname",
            {"type": "string", "value": "example.com"},
        ),
        (
            "({SOME_PROPERTY:'SOME_VALUE'})",
            "(obj) => obj.SOME_PROPERTY",
            {"type": "string", "value": "SOME_VALUE"},
        ),
    ],
)
async def test_remote_reference_argument(
    bidi_session, top_context, setup_expression, function_declaration, expected
):
    remote_value_result = await bidi_session.script.evaluate(
        expression=setup_expression,
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )
    remote_value_handle = remote_value_result.get("handle")

    assert isinstance(remote_value_handle, str)

    result = await bidi_session.script.call_function(
        function_declaration=function_declaration,
        arguments=[{"handle": remote_value_handle}],
        await_promise=True if remote_value_result["type"] == "promise" else False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == expected


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "value_fn, function_declaration",
    [
        (
            lambda value: value,
            "function(arg) { return arg === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "object", "value": [["nested", value]]}),
            "function(arg) { return arg.nested === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "array", "value": [value]}),
            "function(arg) { return arg[0] === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "map", "value": [["foobar", value]]}),
            "function(arg) { return arg.get('foobar') === window.SOME_OBJECT; }",
        ),
        (
            lambda value: ({"type": "set", "value": [value]}),
            "function(arg) { return arg.has(window.SOME_OBJECT); }",
        ),
    ],
)
async def test_remote_reference_deserialization(
    bidi_session, top_context, call_function, evaluate, value_fn, function_declaration
):
    remote_value = await evaluate(
        "window.SOME_OBJECT = { SOME_PROPERTY: 'SOME_VALUE' }; window.SOME_OBJECT",
        result_ownership="root",
    )

    # Check that a remote value can be successfully deserialized as an "argument"
    # parameter and compared against the original object in the page.
    result = await call_function(
        function_declaration=function_declaration,
        arguments=[value_fn(remote_value)],
    )
    assert result == {"type": "boolean", "value": True}

    # Reload the page to cleanup the state
    await bidi_session.browsing_context.navigate(
        context=top_context["context"], url=top_context["url"], wait="complete"
    )


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "setup_expression, expected_node_type",
    [
        ("document.querySelector('img')", 1),
        ("document.querySelector('input#button').attributes[0]", 2),
        ("document.querySelector('#with-text-node').childNodes[0]", 3),
        ("""document.createProcessingInstruction("xml-stylesheet", "href='foo.css'")""", 7),
        ("document.querySelector('#with-comment').childNodes[0]", 8),
        ("document", 9),
        ("document.doctype", 10),
        ("document.createDocumentFragment()", 11),
        ("document.querySelector('#custom-element').shadowRoot", 11),
    ],
    ids=[
        "element",
        "attribute",
        "text node",
        "processing instruction",
        "comment",
        "document",
        "doctype",
        "document fragment",
        "shadow root",
    ]
)
async def test_remote_reference_node_argument(
    bidi_session, get_test_page, top_context, setup_expression, expected_node_type
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    remote_reference = await bidi_session.script.evaluate(
        expression=setup_expression,
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    result = await bidi_session.script.call_function(
        function_declaration="(node) => node.nodeType",
        arguments=[remote_reference],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "number", "value": expected_node_type}


@pytest.mark.asyncio
async def test_remote_reference_node_cdata(bidi_session, inline, top_context):
    xml_page = inline("""<foo>CDATA section: <![CDATA[ < > & ]]>.</foo>""", doctype="xml")

    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=xml_page, wait="complete"
    )

    remote_reference = await bidi_session.script.evaluate(
        expression="document.querySelector('foo').childNodes[1]",
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    result = await bidi_session.script.call_function(
        function_declaration="(node) => node.nodeType",
        arguments=[remote_reference],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "number", "value": 4}


@pytest.mark.asyncio
async def test_remote_reference_sharedId_precedence_over_handle(
    bidi_session, get_test_page, top_context
):
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=get_test_page(), wait="complete"
    )

    remote_reference = await bidi_session.script.evaluate(
        expression="document.querySelector('img')",
        await_promise=False,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
    )

    assert "handle" in remote_reference
    # Invalidate shared reference to trigger a "no such node" error
    remote_reference["sharedId"] = "foo"

    with pytest.raises(error.NoSuchNodeException):
        await bidi_session.script.call_function(
            function_declaration="(node) => node.nodeType",
            arguments=[remote_reference],
            await_promise=False,
            target=ContextTarget(top_context["context"]),
        )


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "expression, function_declaration, expected",
    [
        (
            "document.getElementsByTagName('span')",
            "(collection) => collection.item(0)",
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {},
                    "childNodeCount": 0,
                    "children": [],
                    "localName": "span",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1
                }
            }
        ),
        (
            "document.querySelectorAll('span')",
            "(nodeList) => nodeList.item(0)",
            {
                "type": "node",
                "sharedId": any_string,
                "value": {
                    "attributes": {},
                    "childNodeCount": 0,
                    "children": [],
                    "localName": "span",
                    "namespaceURI": "http://www.w3.org/1999/xhtml",
                    "nodeType": 1
                }
            }
        ),
    ], ids=[
        "htmlcollection",
        "nodelist"
    ]
)
async def test_remote_reference_dom_collection(
    bidi_session,
    inline,
    top_context,
    call_function,
    expression,
    function_declaration,
    expected
):
    page_url = inline("""<p><span>""")
    await bidi_session.browsing_context.navigate(
        context=top_context['context'], url=page_url, wait="complete"
    )

    remote_value = await bidi_session.script.evaluate(
        expression=expression,
        result_ownership="root",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    # Check that a remote value can be successfully deserialized as an "argument"
    # parameter and the first element be extracted.
    result = await call_function(
        function_declaration=function_declaration,
        arguments=[remote_value],
        serialization_options=SerializationOptions(max_dom_depth=1),
    )

    recursive_compare(expected, result)


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "channel, expected_data",
    [
        (
            {"type": "channel", "value": {"channel": "channel_name"}},
            {
                "type": "object",
                "value": [
                    ["foo", {"type": "string", "value": "bar"}],
                    [
                        "baz",
                        {
                            "type": "object",
                            "value": [["1", {"type": "number", "value": 2}]],
                        },
                    ],
                ],
            },
        ),
        (
            {
                "type": "channel",
                "value": {
                    "channel": "channel_name",
                    "serializationOptions": {
                        "maxObjectDepth": 0
                    },
                },
            },
            {"type": "object"},
        ),
        (
            {
                "type": "channel",
                "value": {"channel": "channel_name", "ownership": "root"},
            },
            {
                "handle": any_string,
                "type": "object",
                "value": [
                    ["foo", {"type": "string", "value": "bar"}],
                    [
                        "baz",
                        {
                            "type": "object",
                            "value": [["1", {"type": "number", "value": 2}]],
                        },
                    ],
                ],
            },
        ),
    ],
    ids=["default", "with serializationOptions", "with ownership"],
)
async def test_channel(
    bidi_session, top_context, subscribe_events, wait_for_event, channel, expected_data
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(channel) => channel({'foo': 'bar', 'baz': {'1': 2}})""",
        arguments=[channel],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )
    event_data = await on_script_message

    recursive_compare(
        {
            "channel": "channel_name",
            "data": expected_data,
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        event_data,
    )


@pytest.mark.asyncio
async def test_channel_with_multiple_arguments(
    bidi_session, top_context, subscribe_events, wait_for_event
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(channel) => channel('will_be_send', 'will_be_ignored')""",
        arguments=[{"type": "channel", "value": {"channel": "channel_name"}}],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    event_data = await on_script_message

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {"type": "string", "value": "will_be_send"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        event_data,
    )


@pytest.mark.asyncio
async def test_two_channels(
    bidi_session,
    top_context,
    subscribe_events,
):
    await subscribe_events(["script.message"])

    # Track all received script.message events in the events array
    events = []

    async def on_event(method, data):
        events.append(data)

    remove_listener = bidi_session.add_event_listener("script.message", on_event)

    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(channel_1, channel_2) => {
            channel_1('message_from_channel_1');
            channel_2('message_from_channel_2')
        }""",
        arguments=[
            {"type": "channel", "value": {"channel": "channel_name_1"}},
            {"type": "channel", "value": {"channel": "channel_name_2"}},
        ],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )

    # Wait for both events
    wait = AsyncPoll(bidi_session, timeout=0.5)
    await wait.until(lambda _: len(events) == 2)

    recursive_compare(
        {
            "channel": "channel_name_1",
            "data": {"type": "string", "value": "message_from_channel_1"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        events[0],
    )

    recursive_compare(
        {
            "channel": "channel_name_2",
            "data": {"type": "string", "value": "message_from_channel_2"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        events[1],
    )

    remove_listener()


@pytest.mark.asyncio
async def test_channel_and_nonchannel_arguments(
    bidi_session,
    top_context,
    wait_for_event,
    subscribe_events,
):
    await subscribe_events(["script.message"])

    on_script_message = wait_for_event("script.message")
    result = await bidi_session.script.call_function(
        raw_result=True,
        function_declaration="""(string, channel) => {
            channel(string);
        }""",
        arguments=[
            {"type": "string", "value": "foo"},
            {"type": "channel", "value": {"channel": "channel_name"}},
        ],
        await_promise=False,
        target=ContextTarget(top_context["context"]),
    )
    event_data = await on_script_message

    recursive_compare(
        {
            "channel": "channel_name",
            "data": {"type": "string", "value": "foo"},
            "source": {
                "realm": result["realm"],
                "context": top_context["context"],
            },
        },
        event_data,
    )
