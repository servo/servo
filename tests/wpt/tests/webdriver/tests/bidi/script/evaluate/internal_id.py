import pytest

from ... import recursive_compare, any_string


@pytest.mark.asyncio
@pytest.mark.parametrize(
    "return_structure, result_type",
    [
        ("[data, data]", "array"),
        ("new Map([['foo', data],['bar', data]])", "map"),
        ("({ 'foo': data, 'bar': data })", "object"),
    ],
)
@pytest.mark.parametrize(
    "expression, type",
    [
        ("[1]", "array"),
        ("new Map([[true, false]])", "map"),
        ("new Set(['baz'])", "set"),
        ("{ baz: 'qux' }", "object"),
    ],
)
async def test_remote_values_with_internal_id(
    evaluate, return_structure, result_type, expression, type
):
    result = await evaluate(f"{{const data = {expression}; {return_structure}}}")
    result_value = result["value"]

    assert len(result_value) == 2

    if result_type == "array":
        value = [
            {"type": type, "internalId": any_string},
            {"type": type, "internalId": any_string},
        ]
        internalId1 = result_value[0]["internalId"]
        internalId2 = result_value[1]["internalId"]
    else:
        value = [
            ["foo", {"type": type, "internalId": any_string}],
            ["bar", {"type": type, "internalId": any_string}],
        ]
        internalId1 = result_value[0][1]["internalId"]
        internalId2 = result_value[1][1]["internalId"]

    # Make sure that the same duplicated objects have the same internal ids
    assert internalId1 == internalId2

    recursive_compare(value, result_value)


@pytest.mark.asyncio
async def test_different_remote_values_have_unique_internal_ids(evaluate):
    result = await evaluate(
        "{const obj1 = [1]; const obj2 = {'foo': 'bar'}; [obj1, obj2, obj1, obj2]}"
    )

    assert len(result["value"]) == 4

    internalId1 = result["value"][0]["internalId"]
    internalId2 = result["value"][1]["internalId"]

    # Make sure that different duplicated objects have different internal ids
    assert internalId1 != internalId2
