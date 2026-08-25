def is_element_enabled(session, element_id):
    return session.transport.send(
        "GET",
        "session/{session_id}/element/{element_id}/enabled".format(
            session_id=session.session_id,
            element_id=element_id
        )
    )
