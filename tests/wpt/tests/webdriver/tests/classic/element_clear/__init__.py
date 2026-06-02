def element_clear(session, element):
    return session.transport.send(
        "POST", "/session/{session_id}/element/{element_id}/clear".format(
            session_id=session.session_id,
            element_id=element.id))
