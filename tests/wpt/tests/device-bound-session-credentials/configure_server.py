import json
import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    request_body = json.loads(request.body.decode('utf-8'))
    session_manager.find_for_request(request).configure_state_for_test(request_body)
    return (200, response.headers, "")
