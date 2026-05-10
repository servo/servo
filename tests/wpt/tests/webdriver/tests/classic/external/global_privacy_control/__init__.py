def get_gpc(session):
    """Get the current Global Privacy Control state.

    Uses the WebDriver '/session/{session_id}/privacy' endpoint.

    :param session: WebDriver session object.

    :returns: Response from the WebDriver endpoint.
    """
    return session.transport.send(
        "GET", "/session/{session_id}/privacy".format(session_id=session.session_id)
    )


def set_gpc(session, value):
    """Set the Global Privacy Control state.

    Uses the WebDriver '/session/{session_id}/privacy' endpoint.

    :param session: WebDriver session object.
    :param value: Boolean value to set for GPC (True to enable, False to disable).

    :returns: Response from the WebDriver endpoint.
    """
    return session.transport.send(
        "POST",
        "/session/{session_id}/privacy".format(session_id=session.session_id),
        {"gpc": value},
    )
