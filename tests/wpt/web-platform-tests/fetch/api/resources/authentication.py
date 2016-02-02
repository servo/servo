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

