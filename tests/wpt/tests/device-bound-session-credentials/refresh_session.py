import importlib
import json
jwt_helper = importlib.import_module('device-bound-session-credentials.jwt_helper')
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    test_session_manager = session_manager.find_for_request(request)
    test_session_manager.set_has_called_refresh(True)

    if test_session_manager.get_refresh_endpoint_unavailable():
        return (500, response.headers, "")

    session_id_header = request.headers.get("Sec-Secure-Session-Id")
    if session_id_header == None:
        return (400, response.headers, "")
    session_id_header = session_id_header.decode('utf-8')
    session_id = int(session_id_header)

    if test_session_manager.get_should_refresh_end_session():
        response_body = {
            "session_identifier": session_id,
            "continue": False
        }
        return (200, response.headers, json.dumps(response_body))

    session_key = test_session_manager.get_session_key(session_id)
    if session_key == None:
        return (400, response.headers, "")

    if test_session_manager.get_refresh_sends_challenge():
        challenge = "refresh_challenge_value"
        if request.headers.get("Secure-Session-Response") == None:
            return (403, [('Secure-Session-Challenge', f'"{challenge}";id="{session_id}"')], "")

        jwt_header, jwt_payload, verified = jwt_helper.decode_jwt(request.headers.get("Secure-Session-Response").decode('utf-8'), session_key)

        early_challenge = test_session_manager.get_early_challenge(session_id)
        if early_challenge is not None:
            challenge = early_challenge

        if not verified or jwt_payload.get("jti") != challenge:
            return (400, response.headers, "")

    return test_session_manager.get_session_instructions_response(session_id, request)
