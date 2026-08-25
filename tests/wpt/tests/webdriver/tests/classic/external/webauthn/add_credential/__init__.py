def add_credential(session, authenticator_id, credential):
    return session.transport.send(
        "POST",
        f"/session/{session.session_id}/webauthn/authenticator/{authenticator_id}/credential",
        credential,
    )
