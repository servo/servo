from typing import Any, Mapping

from webdriver.bidi.modules.script import ContextTarget

async def get_permission_state(bidi_session, context: Mapping[str, Any], name: str) -> str:
    result = await bidi_session.script.call_function(
        function_declaration="""() => {
          return navigator.permissions.query({ name: '%s' })
            .then(val => val.state, err => err.message)
        }""" % name,
        target=ContextTarget(context["context"]),
        await_promise=True)
    return result["value"]


async def get_context_origin(bidi_session, context: Mapping[str, Any]) -> str:
    result = await bidi_session.script.call_function(
        function_declaration="""() => {
          return window.location.origin;
        }""",
        target=ContextTarget(context["context"]),
        await_promise=False)
    return result["value"]
