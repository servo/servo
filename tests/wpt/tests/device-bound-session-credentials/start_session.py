import importlib
jwt_helper = importlib.import_module('device-bound-session-credentials.jwt_helper')
session_provider = importlib.import_module('device-bound-session-credentials.session_provider')

def main(request, response):
    jwt_header, jwt_payload, verified = jwt_helper.decode_jwt(request.headers.get("Sec-Session-Response").decode('utf-8'))
    session_id = session_provider.create_new_session()
    session_provider.set_session_key(session_id, jwt_payload.get('key'))

    if not verified or jwt_payload.get("jti") != "login_challenge_value":
        return (400, response.headers, "")

    return session_provider.get_session_instructions_response(session_id, request)
