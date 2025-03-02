def main(request, response):
    headers = [('Sec-Session-Registration', '(RS256);challenge="login_challenge_value";path="/device-bound-session-credentials/start_session.py"')]
    return (200, headers, "")
