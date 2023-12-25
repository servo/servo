from urllib.parse import urlencode


def basic_authentication(url, **kwargs):
    query = {}

    return url("/webdriver/tests/support/http_handlers/authentication.py",
               query=urlencode(query),
               **kwargs)


def main(request, response):
    username = request.auth.username
    password = request.auth.password

    expected_username = "user"
    if b"username" in request.GET:
        expected_username = request.GET.first(b"username")

    expected_password = "password"
    if b"password" in request.GET:
        expected_password = request.GET.first(b"password")

    if username == expected_username and password == expected_password:
        return b"Authentication done"

    realm = b"test"
    if b"realm" in request.GET:
        realm = request.GET.first(b"realm")

    return ((401, b"Unauthorized"),
            [(b"WWW-Authenticate", b'Basic realm="' + realm + b'"')],
            f"Please login with credentials '{expected_username}' and '{expected_password}'")
