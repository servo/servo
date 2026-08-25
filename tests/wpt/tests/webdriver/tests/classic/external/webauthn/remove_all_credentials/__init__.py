def remove_all_credentials(session, authenticator_id):
    return session.transport.send(
        "DELETE",
        f"/session/{session.session_id}/webauthn/authenticator/{authenticator_id}/credentials",
    )
