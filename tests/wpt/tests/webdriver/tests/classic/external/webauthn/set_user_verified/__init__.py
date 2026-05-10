def set_user_verified(session, authenticator_id, is_user_verified):
    return session.transport.send(
        "POST",
        f"/session/{session.session_id}/webauthn/authenticator/{authenticator_id}/uv",
        {"isUserVerified": is_user_verified},
    )
