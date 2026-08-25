import json
import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    return (200, response.headers, json.dumps(session_manager.find_for_request(request).pull_server_state()))
