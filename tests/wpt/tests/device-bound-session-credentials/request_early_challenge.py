import json
import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    test_session_manager = session_manager.find_for_request(request)

    use_single_header = json.loads(request.body.decode('utf-8')).get("useSingleHeader")
    if use_single_header is None:
        return (400, response.headers, "")

    headers = []
    if request.headers.get(b"origin") is not None:
        # Some tests (e.g. third-party-registration.https.html) set
        # challenges across origins. Allow cookies so that we can get
        # the session_manager for the request.
        headers = [
            ("Access-Control-Allow-Origin", request.headers.get(b"origin")),
            ("Access-Control-Allow-Credentials", "true"),
        ]

    challenges = []
    for session_id in session_manager.find_for_request(request).get_session_ids():
        early_challenge = test_session_manager.get_early_challenge(session_id)
        if early_challenge is not None:
            challenges.append(("Secure-Session-Challenge", f'"{early_challenge}";id="{session_id}"'))

    if use_single_header:
        combined_challenges = [("Secure-Session-Challenge", ", ".join([challenge[1] for challenge in challenges]))]
        return (200, headers + combined_challenges, "")
    else:
        return (200, headers + challenges, "")
