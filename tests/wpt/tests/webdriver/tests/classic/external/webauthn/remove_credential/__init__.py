def remove_credential(session, authenticator_id, credential_id):
    return session.transport.send(
        "DELETE",
        f"/session/{session.session_id}/webauthn/authenticator/{authenticator_id}/credentials/{credential_id}",
    )
