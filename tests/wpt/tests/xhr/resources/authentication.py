def main(request, response):
    session_user = request.auth.username
    session_pass = request.auth.password
    expected_user_name = request.headers.get(b"X-User", None)

    token = expected_user_name
    if session_user is None and session_pass is None:
        if token is not None and request.server.stash.take(token) is not None:
            return b'FAIL (did not authorize)'
        else:
            if token is not None:
                request.server.stash.put(token, b"1")
            status = (401, b'Unauthorized')
            headers = [(b'WWW-Authenticate', b'Basic realm="test"')]
            return status, headers, b'FAIL (should be transparent)'
    else:
        if request.server.stash.take(token) == b"1":
            challenge = b"DID"
        else:
            challenge = b"DID-NOT"
        headers = [(b'XHR-USER', expected_user_name),
                   (b'SES-USER', session_user),
                   (b"X-challenge", challenge)]
        return headers, session_user + b"\n" + session_pass
