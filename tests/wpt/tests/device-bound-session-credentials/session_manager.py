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
        raise Exception(f"Could not find manager for test_id: {test_id}")
    return manager

class CookieDetail:
    def __init__(self, name_and_value = None, attributes = None):
        self.name_and_value = name_and_value
        self.attributes = attributes

    def get_name_and_value(self):
        if self.name_and_value is None:
            return "auth_cookie=abcdef0123"
        return self.name_and_value

    def get_attributes(self, request):
        if self.attributes is None:
            return f"Domain={request.url_parts.hostname}; Path=/device-bound-session-credentials"
        return self.attributes

class SessionManager:
    def __init__(self):
        self.session_to_key_map = {}
        self.should_refresh_end_session = False
        self.authorization_value = None
        self.scope_origin = None
        self.registration_sends_challenge = False
        self.cookie_details = None
        self.session_to_cookie_details_map = {}
        self.session_to_early_challenge_map = {}
        self.has_called_refresh = False
        self.scope_specification_items = []
        self.refresh_sends_challenge = True
        self.refresh_url = "/device-bound-session-credentials/refresh_session.py"
        self.include_site = True

    def next_session_id(self):
        return len(self.session_to_key_map)

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

        scope_origin = configuration.get("scopeOrigin")
        if scope_origin is not None:
            self.scope_origin = scope_origin

        registration_sends_challenge = configuration.get("registrationSendsChallenge")
        if registration_sends_challenge is not None:
            self.registration_sends_challenge = registration_sends_challenge

        cookie_details = configuration.get("cookieDetails")
        if cookie_details is not None:
            self.cookie_details = []
            for detail in cookie_details:
                self.cookie_details.append(CookieDetail(detail.get("nameAndValue"), detail.get("attributes")))

        next_sessions_cookie_details = configuration.get("cookieDetailsForNextRegisteredSessions")
        if next_sessions_cookie_details is not None:
            next_session_id = self.next_session_id()
            for session in next_sessions_cookie_details:
                self.session_to_cookie_details_map[next_session_id] = []
                for detail in session:
                    self.session_to_cookie_details_map[next_session_id].append(CookieDetail(detail.get("nameAndValue"), detail.get("attributes")))
                next_session_id += 1

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

        include_site = configuration.get("includeSite")
        if include_site is not None:
            self.include_site = include_site

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

    def get_cookie_details(self, session_id):
        # Try to use the session-specific override first.
        if self.session_to_cookie_details_map.get(session_id) is not None:
            return self.session_to_cookie_details_map[session_id]
        # If there isn't any, use the general override.
        if self.cookie_details is not None:
            return self.cookie_details
        return [CookieDetail()]

    def get_early_challenge(self, session_id):
        return self.session_to_early_challenge_map.get(session_id)

    def get_sessions_instructions_response_credentials(self, session_id, request):
        return list(map(lambda cookie_detail: {
            "type": "cookie",
            "name": cookie_detail.get_name_and_value().split("=")[0],
            "attributes": cookie_detail.get_attributes(request)
        }, self.get_cookie_details(session_id)))

    def get_session_instructions_response_set_cookie_headers(self, session_id, request):
        header_values = list(map(
            lambda cookie_detail: f"{cookie_detail.get_name_and_value()}; {cookie_detail.get_attributes(request)}",
            self.get_cookie_details(session_id)
        ))
        return [("Set-Cookie", header_value) for header_value in header_values]

    def get_session_instructions_response(self, session_id, request):
        scope_origin = ""
        if self.scope_origin is not None:
            scope_origin = self.scope_origin

        response_body = {
            "session_identifier": str(session_id),
            "refresh_url": self.refresh_url,
            "scope": {
                "origin": scope_origin,
                "include_site": self.include_site,
                "scope_specification" : self.scope_specification_items + [
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/request_early_challenge.py" },
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/end_session_via_clear_site_data.py" },
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/pull_server_state.py" },
                    { "type": "exclude", "domain": request.url_parts.hostname, "path": "/device-bound-session-credentials/set_cookie.py" },
                ]
            },
            "credentials": self.get_sessions_instructions_response_credentials(session_id, request)
        }
        headers = self.get_session_instructions_response_set_cookie_headers(session_id, request) + [
            ("Content-Type", "application/json"),
            ("Cache-Control", "no-store")
        ]

        return (200, headers, json.dumps(response_body))
