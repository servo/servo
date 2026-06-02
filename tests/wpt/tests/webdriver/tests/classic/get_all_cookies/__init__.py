def get_all_cookies(session):
    return session.transport.send(
        "GET", "/session/{session_id}/cookie".format(**vars(session))
    )
