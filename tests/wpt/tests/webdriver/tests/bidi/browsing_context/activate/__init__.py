from typing import Any, Mapping

from webdriver.bidi.modules.script import ContextTarget


async def is_element_focused(bidi_session, context: Mapping[str, Any], selector: str) -> bool:
    result = await bidi_session.script.call_function(
        function_declaration="""(selector) => {
        return document.querySelector(selector) === document.activeElement;
    }""",
        arguments=[
            {"type": "string", "value": selector},
        ],
        target=ContextTarget(context["context"]),
        await_promise=False)

    return result["value"]
