async def get_url_for_context(bidi_session, context):
    contexts = await bidi_session.browsing_context.get_tree(root=context)

    return contexts[0]["url"]
