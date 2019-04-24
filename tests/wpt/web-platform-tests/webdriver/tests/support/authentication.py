import urllib


def basic_authentication(username=None, password=None, protocol="http"):
    from .fixtures import server_config, url
    build_url = url(server_config())

    query = {}

    return build_url("/webdriver/tests/support/authentication.py",
                     query=urllib.urlencode(query),
                     protocol=protocol)


def main(request, response):
    user = request.auth.username
    password = request.auth.password

    if user == "user" and password == "password":
        return "Authentication done"

    realm = "test"
    if "realm" in request.GET:
        realm = request.GET.first("realm")

    return ((401, "Unauthorized"),
            [("WWW-Authenticate", 'Basic realm="' + realm + '"')],
            "Please login with credentials 'user' and 'password'")
