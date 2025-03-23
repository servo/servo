import json
import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    request_body = json.loads(request.body.decode('utf-8'))

    test_id = request_body.get("testId")
    if test_id is None:
        test_id = session_manager.initialize_test()

    return (200, [("Set-Cookie", f"test_id={test_id}")], "")
