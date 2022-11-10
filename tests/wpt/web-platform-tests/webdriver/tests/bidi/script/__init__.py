from typing import Any, Callable, Mapping

from .. import any_int, any_string, recursive_compare


def assert_handle(obj: Mapping[str, Any], should_contain_handle: bool) -> None:
    if should_contain_handle:
        assert "handle" in obj, f"Result should contain `handle`. Actual: {obj}"
        assert isinstance(obj["handle"], str), f"`handle` should be a string, but was {type(obj['handle'])}"
    else:
        assert "handle" not in obj, f"Result should not contain `handle`. Actual: {obj}"


def specific_error_response(expected_error: Mapping[str, Any]) -> Callable[[Any], None]:
    return lambda actual: recursive_compare(
        {
            "realm": any_string,
            "exceptionDetails": {
                "columnNumber": any_int,
                "exception": expected_error,
                "lineNumber": any_int,
                "stackTrace": any_stack_trace,
                "text": any_string,
            },
        },
        actual)


def any_stack_trace(actual: Any) -> None:
    assert type(actual) is dict
    assert "callFrames" in actual
    assert type(actual["callFrames"]) is list
    for actual_frame in actual["callFrames"]:
        any_stack_frame(actual_frame)


def any_stack_frame(actual: Any) -> None:
    assert type(actual) is dict

    assert "columnNumber" in actual
    any_int(actual["columnNumber"])

    assert "functionName" in actual
    any_string(actual["functionName"])

    assert "lineNumber" in actual
    any_int(actual["lineNumber"])

    assert "url" in actual
    any_string(actual["url"])
