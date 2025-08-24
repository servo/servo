from typing import Any, Callable, Dict, List, Mapping, Literal

from tests.support.sync import AsyncPoll
from webdriver.bidi.modules.script import ContextTarget
from webdriver.bidi.undefined import UNDEFINED


def get_invalid_cases(
        type_: Literal["boolean", "list", "string", "dict", "number"],
        nullable: bool = False) -> List[Any]:
    """
    Returns invalid use cases for the specific type with the given restrictions.
    >>> get_invalid_cases("boolean")
    [42, None, [], 'foo', {}]
    >>> get_invalid_cases("list")
    [42, False, None, 'foo', {}]
    >>> get_invalid_cases("string")
    [42, False, None, [], {}]
    >>> get_invalid_cases("dict")
    [42, False, None, [], 'foo']
    >>> get_invalid_cases("number")
    [False, None, [], 'foo', {}]
    >>> get_invalid_cases("boolean", nullable=True)
    [42, [], 'foo', {}]
    >>> get_invalid_cases("boolean", nullable=False)
    [42, None, [], 'foo', {}]

    >>> get_invalid_cases("invalid_type")
    Traceback (most recent call last):
      ...
    ValueError: Unexpected type: invalid_type

    """
    cases = {
        "boolean": False,
        "list": [],
        "string": 'foo',
        "dict": {},
        "number": 42,
    }

    if type_ not in cases:
        raise ValueError(f"Unexpected type: {type_}")

    result = list(filter(lambda i: i != cases[type_], cases.values()))
    if not nullable:
        result.append(None)

    return sorted(result, key=lambda x: str(x))


# Compares 2 objects recursively.
# Actual value can have more keys as part of the forwards-compat design.
# Expected value can be a value, a callable delegate, asserting the value, or UNDEFINED.
# If expected value is UNDEFINED, the actual value should not be present.
def recursive_compare(expected: Any, actual: Any) -> None:
    if callable(expected):
        expected(actual)
        return

    if isinstance(actual, List) and isinstance(expected, List):
        assert len(expected) == len(actual)
        for index, _ in enumerate(expected):
            recursive_compare(expected[index], actual[index])
        return

    if isinstance(actual, Dict) and isinstance(expected, Dict):

        # Expected keys with UNDEFINED values should not be present.
        unexpected_keys = {key: value for key, value in expected.items() if
                             value is UNDEFINED}.keys()
        assert not (unexpected_keys & actual.keys()), \
            f"Keys should not be present: {unexpected_keys & actual.keys()}"

        expected_keys = expected.keys() - unexpected_keys
        # Actual Mapping can have more keys as part of the forwards-compat design.
        assert (
            expected_keys <= actual.keys()
        ), f"Key set should be present: {set(expected_keys) - set(actual.keys())}"
        for key in expected_keys:
            recursive_compare(expected[key], actual[key])
        return

    assert expected == actual


def any_bool(actual: Any) -> None:
    assert isinstance(actual, bool)


def any_dict(actual: Any) -> None:
    assert isinstance(actual, dict)


def any_int(actual: Any) -> None:
    assert isinstance(actual, int)


def any_number(actual: Any) -> None:
    assert isinstance(actual, int) or isinstance(actual, float)


def any_int_or_null(actual: Any) -> None:
    if actual is not None:
        any_int(actual)


def any_list(actual: Any) -> None:
    assert isinstance(actual, list)


def any_list_or_null(actual: Any) -> None:
    if actual is not None:
        any_list(actual)


def any_positive_int(actual):
    def _(actual: Any) -> None:
        any_int(actual)
        assert actual > 0

    return _


def any_string(actual: Any) -> None:
    assert isinstance(actual, str)


def any_string_or_null(actual: Any) -> None:
    if actual is not None:
        any_string(actual)


def int_interval(start: int, end: int) -> Callable[[Any], None]:
    def _(actual: Any) -> None:
        any_int(actual)
        assert start <= actual <= end

    return _


def number_interval(start: float, end: float) -> Callable[[Any], None]:
    def _(actual: Any) -> None:
        any_number(actual)
        assert start <= actual <= end

    return _


def assert_cookies(cookies, expected_cookies):
    assert len(cookies) == len(expected_cookies)

    expected = sorted(expected_cookies, key=lambda cookie: cookie["name"])
    actual = sorted(cookies, key=lambda cookie: cookie["name"])

    recursive_compare(expected, actual)


def assert_handle(obj: Mapping[str, Any], should_contain_handle: bool) -> None:
    if should_contain_handle:
        assert "handle" in obj, f"Result should contain `handle`. Actual: {obj}"
        assert isinstance(obj["handle"], str), f"`handle` should be a string, but was {type(obj['handle'])}"

        # Recursively check that handle is not found in any of the nested values.
        if "value" in obj:
            value = obj["value"]
            if type(value) is list:
                for v in value:
                    if type(v) is dict:
                        assert_handle(v, False)

            if type(value) is dict:
                for v in value.values():
                    if type(v) is dict:
                        assert_handle(v, False)

    else:
        assert "handle" not in obj, f"Result should not contain `handle`. Actual: {obj}"


async def create_console_api_message(bidi_session, context: Any, text: str):
    await bidi_session.script.call_function(
        function_declaration="""(text) => console.log(text)""",
        arguments=[{"type": "string", "value": text}],
        await_promise=False,
        target=ContextTarget(context["context"]),
    )
    return text


async def get_device_pixel_ratio(bidi_session, context: Any) -> float:
    result = await bidi_session.script.call_function(
        function_declaration="""() => {
        return window.devicePixelRatio;
    }""",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )
    return result["value"]


async def get_element_dimensions(bidi_session, context, element):
    result = await bidi_session.script.call_function(
        arguments=[element],
        function_declaration="""(element) => {
            const rect = element.getBoundingClientRect();
            return { height: rect.height, width: rect.width }
        }""",
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    return remote_mapping_to_dict(result["value"])


async def get_viewport_dimensions(bidi_session, context: Any,
      with_scrollbar: bool = True, quirk_mode: bool = False):
    if with_scrollbar:
        expression = """
            ({
                height: window.innerHeight,
                width: window.innerWidth,
            });
        """
    else:
        # The way the viewport height without the scrollbar can be calculated
        # is different in quirks mode. In quirks mode, the viewport height is
        # the height of the body element, while in standard mode it is the
        # height of the document element.
        element_expression = \
            "document.body" if quirk_mode else "document.documentElement"
        expression = f"""
            ({{
                height: {element_expression}.clientHeight,
                width: {element_expression}.clientWidth,
            }});
        """
    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    return remote_mapping_to_dict(result["value"])


async def get_document_dimensions(bidi_session, context: Any):
    expression = """
        ({
          height: document.documentElement.scrollHeight,
          width: document.documentElement.scrollWidth,
        });
    """
    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    return remote_mapping_to_dict(result["value"])


async def get_context_origin(bidi_session, context: Mapping[str, Any]) -> str:
    result = await bidi_session.script.call_function(
        function_declaration="""() => {
          return window.location.origin;
        }""",
        target=ContextTarget(context["context"]),
        await_promise=False)
    return result["value"]


def remote_mapping_to_dict(js_object) -> Dict:
    obj = {}
    for key, value in js_object:
        if value["type"] != "null":
            obj[key] = value["value"]

    return obj


async def wait_for_bidi_events(bidi_session, events, count, timeout=2, equal_check=True):
    def check_bidi_events(_, events, count):
        assert len(
            events) >= count, f"Did not receive at least {count} BiDi event(s)"

    wait = AsyncPoll(bidi_session, timeout=timeout)
    await wait.until(check_bidi_events, events, count)

    if equal_check:
        assert len(events) == count, f"Did not receive {count} BiDi event(s)"
