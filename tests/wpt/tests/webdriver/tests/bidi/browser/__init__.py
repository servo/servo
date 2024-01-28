async def get_user_context_ids(bidi_session):
    """
    Returns the list of string ids of the current user contexts.
    """
    user_contexts = await bidi_session.browser.get_user_contexts()
    return [user_context_info["userContext"] for user_context_info in user_contexts]
