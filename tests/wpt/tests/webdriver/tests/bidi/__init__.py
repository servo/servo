from typing import Any, Callable, Dict, List, Mapping
from webdriver.bidi.modules.script import ContextTarget


# Compares 2 objects recursively.
# Actual value can have more keys as part of the forwards-compat design.
# Expected value can be a callable delegate, asserting the value.
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
        # Actual Mapping can have more keys as part of the forwards-compat design.
        assert (
            expected.keys() <= actual.keys()
        ), f"Key set should be present: {set(expected.keys()) - set(actual.keys())}"
        for key in expected.keys():
            recursive_compare(expected[key], actual[key])
        return

    assert expected == actual


def any_bool(actual: Any) -> None:
    assert isinstance(actual, bool)


def any_dict(actual: Any) -> None:
    assert isinstance(actual, dict)


def any_int(actual: Any) -> None:
    assert isinstance(actual, int)


def any_int_or_null(actual: Any) -> None:
    if actual is not None:
        any_int(actual)


def any_list(actual: Any) -> None:
    assert isinstance(actual, list)


def any_list_or_null(actual: Any) -> None:
    if actual is not None:
        any_list(actual)


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


async def create_console_api_message(bidi_session, context: str, text: str):
    await bidi_session.script.call_function(
        function_declaration="""(text) => console.log(text)""",
        arguments=[{"type": "string", "value": text}],
        await_promise=False,
        target=ContextTarget(context["context"]),
    )
    return text


async def get_device_pixel_ratio(bidi_session, context: str) -> float:
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


async def get_viewport_dimensions(bidi_session, context: str):
    expression = """
        ({
          height: window.innerHeight || document.documentElement.clientHeight,
          width: window.innerWidth || document.documentElement.clientWidth,
        });
    """
    result = await bidi_session.script.evaluate(
        expression=expression,
        target=ContextTarget(context["context"]),
        await_promise=False,
    )

    return remote_mapping_to_dict(result["value"])


async def get_document_dimensions(bidi_session, context: str):
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


def remote_mapping_to_dict(js_object) -> Dict:
    obj = {}
    for key, value in js_object:
        obj[key] = value["value"]

    return obj
