import pytest
from webdriver.bidi.error import UnknownErrorException
from webdriver.bidi.modules.script import ContextTarget

pytestmark = pytest.mark.asyncio


def nested_arrays_expression(depth):
    return f"(() => {{ let a = []; for (let i = 1; i < {depth}; ++i) a = [a]; return a; }})()"


def nested_objects_expression(depth):
    return f"(() => {{ let a = {{}}; for (let i = 1; i < {depth}; ++i) a = {{ a }}; return a; }})()"


def assert_nested_arrays(result, depth):
    for _ in range(depth - 1):
        assert result["type"] == "array"
        assert len(result["value"]) == 1
        result = result["value"][0]
    assert result == {"type": "array", "value": []}


def assert_nested_objects(result, depth):
    for _ in range(depth - 1):
        assert result["type"] == "object"
        assert len(result["value"]) == 1
        assert result["value"][0][0] == "a"
        result = result["value"][0][1]
    assert result == {"type": "object", "value": []}


async def test_nested_arrays(bidi_session, top_context):
    result = await bidi_session.script.evaluate(
        expression=nested_arrays_expression(100),
        target=ContextTarget(top_context["context"]),
        await_promise=False,
    )

    assert_nested_arrays(result, 100)


@pytest.mark.parametrize(
    "expression, assert_result, depth",
    [
        (nested_arrays_expression, assert_nested_arrays, 500),
        (nested_objects_expression, assert_nested_objects, 400),
    ],
    ids=["arrays", "objects"],
)
async def test_deeply_nested_values(bidi_session, top_context, expression, assert_result, depth):
    # A remote end can fail to serialize a value this deep, but it has to answer the command.
    try:
        result = await bidi_session.script.evaluate(
            expression=expression(depth),
            target=ContextTarget(top_context["context"]),
            await_promise=False,
        )
    except UnknownErrorException:
        return

    assert_result(result, depth)
