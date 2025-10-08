import importlib
from urllib.parse import parse_qs
jwt_helper = importlib.import_module('device-bound-session-credentials.jwt_helper')
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    test_session_manager = session_manager.find_for_request(request)
    extra_cookie_headers = test_session_manager.get_set_cookie_headers(test_session_manager.registration_extra_cookies, request)
    if test_session_manager.get_registration_sends_challenge_before_instructions():
        # Only send back a challenge on the first call.
        test_session_manager.reset_registration_sends_challenge_before_instructions()
        return (403, [('Secure-Session-Challenge', '"login_challenge_value"')] + extra_cookie_headers, "")

    jwt_header, jwt_payload, verified = jwt_helper.decode_jwt(request.headers.get("Secure-Session-Response").decode('utf-8'))
    session_id = test_session_manager.create_new_session()
    test_session_manager.set_session_key(session_id, jwt_header.get('jwk'))

    if not verified or jwt_payload.get("jti") != "login_challenge_value":
        return (400, list(response.headers) + extra_cookie_headers, "")

    if jwt_payload.get("authorization") != test_session_manager.get_authorization_value():
        return (400, list(response.headers) + extra_cookie_headers, "")

    if jwt_payload.get("sub") is not None:
        return (400, list(response.headers) + extra_cookie_headers, "")

    if test_session_manager.get_has_custom_query_param() and 'registrationQueryParam' not in parse_qs(request.url_parts.query):
        return (400, list(response.headers) + extra_cookie_headers, "")

    (code, headers, body) = test_session_manager.get_session_instructions_response(session_id, request)
    headers += extra_cookie_headers
    if test_session_manager.get_registration_sends_challenge_with_instructions():
        headers.append(('Secure-Session-Challenge', f'"login_challenge_value";id="{session_id}"'))
    return (code, headers, body)
