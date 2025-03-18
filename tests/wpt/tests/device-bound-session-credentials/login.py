import json
import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    num_sessions = 1
    use_single_header = True
    registration_url = "/device-bound-session-credentials/start_session.py"
    request_body_raw = request.body.decode('utf-8')
    if len(request_body_raw) > 0:
        request_body = json.loads(request_body_raw)
        maybe_num_sessions = request_body.get("numSessions")
        if maybe_num_sessions is not None:
            num_sessions = maybe_num_sessions
        maybe_use_single_header = request_body.get("useSingleHeader")
        if maybe_use_single_header is not None:
            use_single_header = maybe_use_single_header
        maybe_registration_url = request_body.get("registrationUrl")
        if maybe_registration_url is not None:
            registration_url = maybe_registration_url

    authorization_value = session_manager.find_for_request(request).get_authorization_value()
    authorization_header = ""
    if authorization_value is not None:
        authorization_header = f';authorization="{authorization_value}"'

    registrations = []
    for i in range(num_sessions):
        registrations.append(('Sec-Session-Registration', f'(RS256);challenge="login_challenge_value";path="{registration_url}"{authorization_header}'))

    if use_single_header:
        combined_registrations = [("Sec-Session-Registration", ", ".join([registration[1] for registration in registrations]))]
        return (200, combined_registrations, "")
    else:
        return (200, registrations, "")
