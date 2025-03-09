import importlib
import json
jwt_helper = importlib.import_module('device-bound-session-credentials.jwt_helper')
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    session_id_header = request.headers.get("Sec-Session-Id")
    if session_id_header == None:
        return (400, response.headers, "")
    session_id = session_id_header.decode('utf-8')
    test_session_manager = session_manager.find_for_request(request)

    if test_session_manager.get_should_refresh_end_session():
        response_body = {
            "session_identifier": session_id,
            "continue": False
        }
        return (200, response.headers, json.dumps(response_body))

    session_key = test_session_manager.get_session_key(session_id)
    if session_key == None:
        return (400, response.headers, "")

    challenge = "refresh_challenge_value"
    if request.headers.get("Sec-Session-Response") == None:
        return (401, [('Sec-Session-Challenge', '"' + challenge + '";id="' + session_id + '"')], "")

    jwt_header, jwt_payload, verified = jwt_helper.decode_jwt(request.headers.get("Sec-Session-Response").decode('utf-8'), session_key)

    if test_session_manager.get_send_challenge_early():
        challenge = "early_challenge"

    if not verified or jwt_payload.get("jti") != challenge:
        return (400, response.headers, "")

    return test_session_manager.get_session_instructions_response(session_id, request)
