def set_credential_properties(
    session, authenticator_id, credential_id, properties
):
    return session.transport.send(
        "POST",
        f"/session/{session.session_id}/webauthn/authenticator/{authenticator_id}/credentials/{credential_id}/props",
        properties,
    )
