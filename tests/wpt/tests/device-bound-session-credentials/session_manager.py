import json

test_to_session_manager_mapping = {}

def initialize_test():
    test_id = str(len(test_to_session_manager_mapping))
    test_to_session_manager_mapping[test_id] = SessionManager()
    return test_id

def find_for_request(request):
    test_id = request.cookies.get(b'test_id').value.decode('utf-8')
    manager = test_to_session_manager_mapping.get(test_id)
    if manager == None:
        raise Exception("Could not find manager for test_id: " + test_id)
    return manager

class SessionManager:
    def __init__(self):
        self.session_to_key_map = {}
        self.should_refresh_end_session = False
        self.authorization_value = None
        self.send_challenge_early = False
        self.cookie_has_no_attributes = False
        self.scope_origin = None
        self.registration_sends_challenge = False
        self.cookie_name_and_value = "auth_cookie=abcdef0123"

    def create_new_session(self):
        session_id = str(len(self.session_to_key_map))
        self.session_to_key_map[session_id] = None
        return session_id

    def set_session_key(self, session_id, key):
        if session_id not in self.session_to_key_map:
            return False
        self.session_to_key_map[session_id] = key
        return True

    def get_session_key(self, session_id):
        return self.session_to_key_map.get(session_id)

    def get_session_ids(self):
        return list(self.session_to_key_map.keys())

    def configure_state_for_test(self, configuration):
        should_refresh_end_session = configuration.get("shouldRefreshEndSession")
        if should_refresh_end_session is not None:
            self.should_refresh_end_session = should_refresh_end_session

        authorization_value = configuration.get("authorizationValue")
        if authorization_value is not None:
            self.authorization_value = authorization_value

        send_challenge_early = configuration.get("sendChallengeEarly")
        if send_challenge_early is not None:
            self.send_challenge_early = send_challenge_early

        cookie_has_no_attributes = configuration.get("cookieHasNoAttributes")
        if cookie_has_no_attributes is not None:
            self.cookie_has_no_attributes = cookie_has_no_attributes

        scope_origin = configuration.get("scopeOrigin")
        if scope_origin is not None:
            self.scope_origin = scope_origin

        registration_sends_challenge = configuration.get("registrationSendsChallenge")
        if registration_sends_challenge is not None:
            self.registration_sends_challenge = registration_sends_challenge

        cookie_name_and_value = configuration.get("cookieNameAndValue")
        if cookie_name_and_value is not None:
            self.cookie_name_and_value = cookie_name_and_value

    def get_should_refresh_end_session(self):
        return self.should_refresh_end_session

    def get_authorization_value(self):
        return self.authorization_value

    def get_send_challenge_early(self):
        return self.send_challenge_early

    def get_registration_sends_challenge(self):
        return self.registration_sends_challenge

    def reset_registration_sends_challenge(self):
        self.registration_sends_challenge = False

    def get_session_instructions_response(self, session_id, request):
        cookie_parts = [self.cookie_name_and_value]
        cookie_attributes = ""
        if not self.cookie_has_no_attributes:
            cookie_attributes = "Domain=" + request.url_parts.hostname + "; Path=/device-bound-session-credentials"
            cookie_parts.append(cookie_attributes)
        value_of_set_cookie = "; ".join(cookie_parts)

        scope_origin = ""
        if self.scope_origin is not None:
            scope_origin = self.scope_origin

        response_body = {
            "session_identifier": session_id,
            "refresh_url": "/device-bound-session-credentials/refresh_session.py",
            "scope": {
                "origin": scope_origin,
                "include_site": True,
                "scope_specification" : [
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/request_early_challenge.py" },
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/end_session_via_clear_site_data.py" },
                ]
            },
            "credentials": [{
                "type": "cookie",
                "name": self.cookie_name_and_value.split("=")[0],
                "attributes": cookie_attributes
            }]
        }
        headers = [
            ("Content-Type", "application/json"),
            ("Cache-Control", "no-store"),
            ("Set-Cookie", value_of_set_cookie)
        ]

        return (200, headers, json.dumps(response_body))
