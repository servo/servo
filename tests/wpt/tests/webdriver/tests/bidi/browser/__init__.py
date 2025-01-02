from webdriver.bidi.modules.script import ContextTarget

async def get_user_context_ids(bidi_session):
    """
    Returns the list of string ids of the current user contexts.
    """
    user_contexts = await bidi_session.browser.get_user_contexts()
    return [user_context_info["userContext"] for user_context_info in user_contexts]


async def set_local_storage(bidi_session, context: str, key: str, value: str):
    """
    Sets the value for the key in the context's localStorage.
    """
    await bidi_session.script.call_function(
        function_declaration="""(key, value) => localStorage.setItem(key, value)""",
        arguments=[{"type": "string", "value": key}, {"type": "string", "value": value}],
        await_promise=False,
        target=ContextTarget(context["context"]),
    )


async def get_local_storage(bidi_session, context: str, key: str):
    """
    Returns the value identified by the key from the context's localStorage.
    """
    result = await bidi_session.script.call_function(
        function_declaration="""(key) => localStorage.getItem(key)""",
        arguments=[{"type": "string", "value": key}],
        await_promise=False,
        target=ContextTarget(context["context"]),
    )
    if not "value" in result:
        return None
    return result["value"]
