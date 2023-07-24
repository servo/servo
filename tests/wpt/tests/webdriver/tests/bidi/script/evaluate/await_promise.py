import pytest

from webdriver.bidi.modules.script import ContextTarget, ScriptEvaluateResultException

from ... import any_int, any_string, recursive_compare
from .. import any_stack_trace, PRIMITIVE_VALUES


@pytest.mark.asyncio
async def test_await_promise_delayed(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="""
          new Promise(r => {{
            setTimeout(() => r("SOME_DELAYED_RESULT"), 0);
          }})
        """,
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {"type": "string", "value": "SOME_DELAYED_RESULT"}


@pytest.mark.asyncio
async def test_await_promise_rejected(bidi_session, top_context):
    with pytest.raises(ScriptEvaluateResultException) as exception:
        await bidi_session.script.evaluate(
            expression="Promise.reject('SOME_REJECTED_RESULT')",
            target=ContextTarget(top_context["context"]),
            await_promise=True,
        )

    recursive_compare(
        {
            "realm": any_string,
            "exceptionDetails": {
                "columnNumber": any_int,
                "exception": {"type": "string", "value": "SOME_REJECTED_RESULT"},
                "lineNumber": any_int,
                "stackTrace": any_stack_trace,
                "text": any_string,
            },
        },
        exception.value.result,
    )


@pytest.mark.asyncio
async def test_await_promise_resolved(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="Promise.resolve('SOME_RESOLVED_RESULT')",
        target=ContextTarget(top_context["context"]),
        await_promise=True,
    )

    assert result == {"type": "string", "value": "SOME_RESOLVED_RESULT"}


@pytest.mark.asyncio
async def test_await_resolve_array(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="Promise.resolve([1, 'text', true, ['will be serialized']])",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {
        "type": "array",
        "value": [
            {"type": "number", "value": 1},
            {"type": "string", "value": "text"},
            {"type": "boolean", "value": True},
            {"type": "array", "value": [{"type": "string", "value": "will be serialized"}]},
        ],
    }


@pytest.mark.asyncio
async def test_await_resolve_date(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="Promise.resolve(new Date(0))",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {
        "type": "date",
        "value": "1970-01-01T00:00:00.000Z",
    }


@pytest.mark.asyncio
async def test_await_resolve_map(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="""
        Promise.resolve(
            new Map([
                ['key1', 'value1'],
                [2, new Date(0)],
                ['key3', new Map([['key4', 'serialized']])]
            ])
        )""",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {
        "type": "map",
        "value": [
            ["key1", {"type": "string", "value": "value1"}],
            [
                {"type": "number", "value": 2},
                {"type": "date", "value": "1970-01-01T00:00:00.000Z"},
            ],
            ["key3", {"type": "map", "value": [[
                "key4",
                {"type": "string", "value": "serialized"}
            ]]}],
        ],
    }


@pytest.mark.parametrize("expression, expected", PRIMITIVE_VALUES)
@pytest.mark.asyncio
async def test_await_resolve_primitive(
    bidi_session, top_context, expression, expected
):
    result = await bidi_session.script.evaluate(
        expression=f"Promise.resolve({expression})",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == expected


@pytest.mark.asyncio
async def test_await_resolve_regexp(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="Promise.resolve(/test/i)",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {
        "type": "regexp",
        "value": {
            "pattern": "test",
            "flags": "i",
        },
    }


@pytest.mark.asyncio
async def test_await_resolve_set(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="""
        Promise.resolve(
            new Set([
                'value1',
                2,
                true,
                new Date(0),
                new Set([-1, 'serialized'])
            ])
        )""",
        await_promise=True,
        target=ContextTarget(top_context["context"]),
    )

    assert result == {
        "type": "set",
        "value": [
            {"type": "string", "value": "value1"},
            {"type": "number", "value": 2},
            {"type": "boolean", "value": True},
            {"type": "date", "value": "1970-01-01T00:00:00.000Z"},
            {"type": "set", "value": [{"type": "number", "value": -1}, {"type": "string", "value": "serialized"}]},
        ],
    }


@pytest.mark.asyncio
async def test_no_await_promise_rejected(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="Promise.reject('SOME_REJECTED_RESULT')",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare({"type": "promise"}, result)


@pytest.mark.asyncio
async def test_no_await_promise_resolved(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression="Promise.resolve('SOME_RESOLVED_RESULT')",
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    recursive_compare({"type": "promise"}, result)
