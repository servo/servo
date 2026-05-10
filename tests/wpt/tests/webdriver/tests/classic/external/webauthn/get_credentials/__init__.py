def get_credentials(session, authenticator_id):
    return session.transport.send(
        "GET",
        f"/session/{session.session_id}/webauthn/authenticator/{authenticator_id}/credentials",
    )
