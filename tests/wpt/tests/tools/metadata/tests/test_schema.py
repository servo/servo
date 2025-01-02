# mypy: allow-untyped-defs

from ..schema import SchemaValue, validate_dict
from dataclasses import dataclass, asdict

import pytest
import re

@pytest.mark.parametrize(
    "input,kwargs,expected_result,expected_exception_type,exception_message",
    [
        ({}, {}, None, None, None),
        ("2", {}, None, ValueError, "Object is not a dictionary. Input: 2"),
        ({"extra": 3}, {}, None, ValueError, "Object contains invalid keys: ['extra']"),
        ({"required": 1}, {"required_keys": {"required"}}, None, None, None),
        ({"optional": 2}, {"optional_keys": {"optional"}}, None, None, None),
        ({"extra": 3, "optional": 2}, {"optional_keys": {"optional"}}, None,
            ValueError, "Object contains invalid keys: ['extra']"),
        ({"required": 1, "optional": 2}, {"required_keys": {"required"}, "optional_keys": {"optional"}}, None, None, None),
        ({"optional": 2}, {"required_keys": {"required"}, "optional_keys": {"optional"}}, None,
            ValueError, "Object missing required keys: ['required']"),
    ])
def test_validate_dict(input, kwargs, expected_result, expected_exception_type, exception_message):
    if expected_exception_type:
        with pytest.raises(expected_exception_type, match=re.escape(exception_message)):
            validate_dict(input, **kwargs)
    else:
        expected_result == validate_dict(input, **kwargs)


@dataclass
class FromDictTestDataClass:
    key: str

    def __init__(self, input):
        self.key = input.get("key")

@pytest.mark.parametrize(
    "input,expected_result,expected_exception_type,exception_message",
    [
        ({"key": "value"}, {"key": "value"}, None, None),
        ({1: "value"}, None, ValueError, "Input value {1: 'value'} contains key 1 that is not a string"),
        (3, None, ValueError, "Input value 3 is not a dict")
    ])
def test_from_dict(input, expected_result, expected_exception_type, exception_message):
    if expected_exception_type:
        with pytest.raises(expected_exception_type, match=exception_message):
            FromDictTestDataClass(SchemaValue.from_dict(input))
    else:
        assert expected_result == asdict(FromDictTestDataClass(SchemaValue.from_dict(input)))


@pytest.mark.parametrize(
    "input,expected_result,expected_exception_type,exception_message",
    [
        ("test", "test", None, None),
        (2, None, ValueError, "Input value 2 is not a string")
    ])
def test_from_str(input, expected_result, expected_exception_type, exception_message):
    if expected_exception_type:
        with pytest.raises(expected_exception_type, match=exception_message):
            SchemaValue.from_str(input)
    else:
        assert expected_result == SchemaValue.from_str(input)


@pytest.mark.parametrize(
    "input,expected_result,expected_exception_type,exception_message",
    [
        (["1", "2"], ["1", "2"], None, None),
        (2, None, ValueError, "Input value 2 is not a list")
    ])
def test_from_list(input, expected_result, expected_exception_type, exception_message):
    if expected_exception_type:
        with pytest.raises(expected_exception_type, match=exception_message):
            SchemaValue.from_list(SchemaValue.from_str, input)
    else:
        assert expected_result == SchemaValue.from_list(SchemaValue.from_str, input)


@pytest.mark.parametrize(
    "input,expected_result,expected_exception_type,exception_message",
    [
        ("test", "test", None, None),
        (None, None, None, None),
        (2, None, ValueError, "Input value 2 does not fit one of the expected values for the union")
    ])
def test_from_union(input,expected_result, expected_exception_type, exception_message):
    union_input = [SchemaValue.from_str, SchemaValue.from_none]
    if expected_exception_type:
        with pytest.raises(expected_exception_type, match=exception_message):
            SchemaValue.from_union(union_input, input)
    else:
        assert expected_result == SchemaValue.from_union(union_input, input)
