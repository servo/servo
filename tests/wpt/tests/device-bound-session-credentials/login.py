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

    test_session_manager = session_manager.find_for_request(request)

    header_items = ["(RS256)",'challenge="login_challenge_value"',f'path="{registration_url}"']
    authorization_value = test_session_manager.get_authorization_value()
    if authorization_value is not None:
        header_items.append(f'authorization="{authorization_value}"')
    provider_session_id = test_session_manager.get_provider_session_id()
    if provider_session_id is not None:
        header_items.append(f'provider_session_id="{provider_session_id}"')
    provider_url = test_session_manager.get_provider_url()
    if provider_url is not None:
        header_items.append(f'provider_url="{provider_url}"')
    provider_key = test_session_manager.get_provider_key()
    if provider_key is not None:
        header_items.append(f'provider_key="{provider_key}"')

    registrations = []
    for i in range(num_sessions):
        registrations.append(('Secure-Session-Registration', ";".join(header_items)))

    headers = []
    if request.headers.get(b"origin") is not None:
        # Some tests (e.g. subdomain-registration.https.html) login
        # across origins. Allow cookies so that we can get the
        # session_manager for the request.
        headers = [
            ("Access-Control-Allow-Origin", request.headers.get(b"origin")),
            ("Access-Control-Allow-Credentials", "true"),
        ]

    if use_single_header:
        combined_registrations = [("Secure-Session-Registration", ", ".join([registration[1] for registration in registrations]))]
        return (200, headers + combined_registrations, "")
    else:
        return (200, headers + registrations, "")
