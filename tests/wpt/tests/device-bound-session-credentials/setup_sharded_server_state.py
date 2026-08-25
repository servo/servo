import json
import importlib
session_manager = importlib.import_module('device-bound-session-credentials.session_manager')

def main(request, response):
    request_body = json.loads(request.body.decode('utf-8'))

    test_id = request_body.get("testId")
    if test_id is None:
        test_id = session_manager.initialize_test()

    # subdomain-registration.https.html does registration on a
    # subdomain. Without the Domain attribute, the test_id cookie won't
    # be available on the subdomain.
    cookie_attributes = f"Domain={request.url_parts.hostname}"

    # Cross-site tests (e.g. allowed-refresh-initiators.https.html) require a
    # SameSite=None cookie, which must also be Secure. But
    # not-secure-connection.html cannot have a Secure cookie, so we need to make
    # the attributes conditional on the test.
    cross_site = request_body.get("crossSite")
    if cross_site is not None and cross_site:
        cookie_attributes += ";SameSite=None;Secure"

    headers = [("Set-Cookie", f"test_id={test_id};{cookie_attributes}")]

    return (200, headers, "")
