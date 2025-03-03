import json

session_to_key_map = {}

def create_new_session():
    session_id = str(len(session_to_key_map))
    session_to_key_map[session_id] = None
    return session_id

def set_session_key(session_id, key):
    if session_id not in session_to_key_map:
        return False
    session_to_key_map[session_id] = key
    return True

def get_session_key(session_id):
    return session_to_key_map.get(session_id)

def clear_server_state():
    global session_to_key_map
    session_to_key_map = {}

def get_session_instructions_response(session_id, request):
    refresh_url = "/device-bound-session-credentials/refresh_session.py"

    response_body = {
        "session_identifier": session_id,
        "refresh_url": refresh_url,
        "scope": {
            "include_site": True,
            "scope_specification" : [
                { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/clear_server_state_and_end_sessions.py" },
            ]
        },
        "credentials": [{
            "type": "cookie",
            "name": "auth_cookie",
            "attributes": "Domain=" + request.url_parts.hostname + "; Path=/device-bound-session-credentials"
        }]
    }
    headers = [
        ("Content-Type", "application/json"),
        ("Cache-Control", "no-store"),
        ("Set-Cookie", "auth_cookie=abcdef0123; Domain=" + request.url_parts.hostname + "; Path=/device-bound-session-credentials")
    ]
    return (200, headers, json.dumps(response_body))
