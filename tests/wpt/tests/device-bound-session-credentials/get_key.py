import importlib
import json
jwt_helper = importlib.import_module('device-bound-session-credentials.jwt_helper')
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    test_session_manager = session_manager.find_for_request(request)
    key = test_session_manager.get_session_key(int(request.url_parts.query))
    return (200, [], jwt_helper.thumbprint_for_jwk(key))

