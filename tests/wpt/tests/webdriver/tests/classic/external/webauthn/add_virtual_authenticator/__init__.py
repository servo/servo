def add_virtual_authenticator(session, config):
    return session.transport.send(
        "POST",
        f"/session/{session.session_id}/webauthn/authenticator",
        config,
    )
