import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    authorization_value = session_manager.find_for_request(request).get_authorization_value()
    authorization_header = ""
    if authorization_value is not None:
        authorization_header = ';authorization="' + authorization_value + '"'

    headers = [('Sec-Session-Registration', '(RS256);challenge="login_challenge_value";path="/device-bound-session-credentials/start_session.py"' + authorization_header)]
    return (200, headers, "")
