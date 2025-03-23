import json
import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    test_session_manager = session_manager.find_for_request(request)

    use_single_header = json.loads(request.body.decode('utf-8')).get("useSingleHeader")
    if use_single_header is None:
        return (400, response.headers, "")

    challenges = []
    for session_id in session_manager.find_for_request(request).get_session_ids():
        early_challenge = test_session_manager.get_early_challenge(session_id)
        if early_challenge is not None:
            challenges.append(("Sec-Session-Challenge", f'"{early_challenge}";id="{session_id}"'))

    if use_single_header:
        combined_challenges = [("Sec-Session-Challenge", ", ".join([challenge[1] for challenge in challenges]))]
        return (200, combined_challenges, "")
    else:
        return (200, challenges, "")
