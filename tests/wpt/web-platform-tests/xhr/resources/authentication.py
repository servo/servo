def main(request, response):
    session_user = request.auth.username
    session_pass = request.auth.password
    expected_user_name = request.headers.get("X-User", None)

    token = expected_user_name
    if session_user is None and session_pass is None:
        if token is not None and request.server.stash.take(token) is not None:
            return 'FAIL (did not authorize)'
        else:
            if token is not None:
                request.server.stash.put(token, "1")
            status = (401, 'Unauthorized')
            headers = [('WWW-Authenticate', 'Basic realm="test"')]
            return status, headers, 'FAIL (should be transparent)'
    else:
        if request.server.stash.take(token) == "1":
            challenge = "DID"
        else:
            challenge = "DID-NOT"
        headers = [('XHR-USER', expected_user_name),
                   ('SES-USER', session_user),
                   ("X-challenge", challenge)]
        return headers, session_user + "\n" + session_pass
