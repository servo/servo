from urllib.parse import urlencode


def basic_authentication(url, username=None, password=None, protocol="http"):
    query = {}

    return url("/webdriver/tests/support/http_handlers/authentication.py",
               query=urlencode(query),
               protocol=protocol)


def main(request, response):
    user = request.auth.username
    password = request.auth.password

    if user == b"user" and password == b"password":
        return b"Authentication done"

    realm = b"test"
    if b"realm" in request.GET:
        realm = request.GET.first(b"realm")

    return ((401, b"Unauthorized"),
            [(b"WWW-Authenticate", b'Basic realm="' + realm + b'"')],
            b"Please login with credentials 'user' and 'password'")
