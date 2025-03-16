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
        self.cookie_attributes = None
        self.scope_origin = None
        self.registration_sends_challenge = False
        self.cookie_name_and_value = "auth_cookie=abcdef0123"
        self.session_to_cookie_name_and_value_map = {}
        self.session_to_early_challenge_map = {}
        self.has_called_refresh = False
        self.scope_specification_items = []
        self.refresh_sends_challenge = True
        self.refresh_url = "/device-bound-session-credentials/refresh_session.py"

    def next_session_id_value(self):
        return len(self.session_to_key_map)
    def next_session_id(self):
        return str(self.next_session_id_value())

    def create_new_session(self):
        session_id = self.next_session_id()
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

        cookie_attributes = configuration.get("cookieAttributes")
        if cookie_attributes is not None:
            self.cookie_attributes = cookie_attributes

        scope_origin = configuration.get("scopeOrigin")
        if scope_origin is not None:
            self.scope_origin = scope_origin

        registration_sends_challenge = configuration.get("registrationSendsChallenge")
        if registration_sends_challenge is not None:
            self.registration_sends_challenge = registration_sends_challenge

        cookie_name_and_value = configuration.get("cookieNameAndValue")
        if cookie_name_and_value is not None:
            self.cookie_name_and_value = cookie_name_and_value

        next_sessions_cookie_names_and_values = configuration.get("cookieNamesAndValuesForNextRegisteredSessions")
        if next_sessions_cookie_names_and_values is not None:
            next_session_id_value = self.next_session_id_value()
            for cookie_name_and_value in next_sessions_cookie_names_and_values:
                self.session_to_cookie_name_and_value_map[str(next_session_id_value)] = cookie_name_and_value
                next_session_id_value += 1

        next_session_early_challenge = configuration.get("earlyChallengeForNextRegisteredSession")
        if next_session_early_challenge is not None:
            self.session_to_early_challenge_map[self.next_session_id()] = next_session_early_challenge

        scope_specification_items = configuration.get("scopeSpecificationItems")
        if scope_specification_items is not None:
            self.scope_specification_items = scope_specification_items

        refresh_sends_challenge = configuration.get("refreshSendsChallenge")
        if refresh_sends_challenge is not None:
            self.refresh_sends_challenge = refresh_sends_challenge

        refresh_url = configuration.get("refreshUrl")
        if refresh_url is not None:
            self.refresh_url = refresh_url

    def get_should_refresh_end_session(self):
        return self.should_refresh_end_session

    def get_authorization_value(self):
        return self.authorization_value

    def get_registration_sends_challenge(self):
        return self.registration_sends_challenge

    def reset_registration_sends_challenge(self):
        self.registration_sends_challenge = False

    def get_refresh_sends_challenge(self):
        return self.refresh_sends_challenge

    def set_has_called_refresh(self, has_called_refresh):
        self.has_called_refresh = has_called_refresh

    def pull_server_state(self):
        return {
            "hasCalledRefresh": self.has_called_refresh
        }

    def get_cookie_name_and_value(self, session_id):
        # Try to use the session-specific override first.
        if self.session_to_cookie_name_and_value_map.get(session_id) is not None:
            return self.session_to_cookie_name_and_value_map[session_id]
        # If there isn't any, use the general override.
        return self.cookie_name_and_value

    def get_early_challenge(self, session_id):
        return self.session_to_early_challenge_map.get(session_id)

    def get_session_instructions_response(self, session_id, request):
        cookie_parts = [self.get_cookie_name_and_value(session_id)]
        cookie_attributes = self.cookie_attributes
        if cookie_attributes is None:
            cookie_attributes = "Domain=" + request.url_parts.hostname + "; Path=/device-bound-session-credentials"
        cookie_parts.append(cookie_attributes)
        value_of_set_cookie = "; ".join(cookie_parts)

        scope_origin = ""
        if self.scope_origin is not None:
            scope_origin = self.scope_origin

        response_body = {
            "session_identifier": session_id,
            "refresh_url": self.refresh_url,
            "scope": {
                "origin": scope_origin,
                "include_site": True,
                "scope_specification" : self.scope_specification_items + [
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/request_early_challenge.py" },
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/end_session_via_clear_site_data.py" },
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/pull_server_state.py" },
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/set_cookie.py" },
                ]
            },
            "credentials": [{
                "type": "cookie",
                "name": self.get_cookie_name_and_value(session_id).split("=")[0],
                "attributes": cookie_attributes
            }]
        }
        headers = [
            ("Content-Type", "application/json"),
            ("Cache-Control", "no-store"),
            ("Set-Cookie", value_of_set_cookie)
        ]

        return (200, headers, json.dumps(response_body))
