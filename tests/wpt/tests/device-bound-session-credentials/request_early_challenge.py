import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    session_ids = session_manager.find_for_request(request).get_session_ids()
    if len(session_ids) != 1:
        return (500, "", "")
    session_id_header = ';id="' + session_ids[0] + '"'

    return (200, [("Sec-Session-Challenge", '"early_challenge"' + session_id_header)], "")
