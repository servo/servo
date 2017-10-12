def main(request, response):
    response.headers.set("Cache-Control", "no-store")
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin"))
    response.headers.set("Access-Control-Allow-Credentials", "true")
    uid = request.GET.first("uid", None)

    if request.method == "OPTIONS":
        response.headers.set("Access-Control-Allow-Methods", "PUT")
    else:
        username = request.auth.username
        password = request.auth.password
        if (not username) or (username != uid):
            response.headers.set("WWW-Authenticate", "Basic realm='Test Realm/Cross Origin'")
            response.status = 401
            response.content = "Authentication cancelled"
        else:
            response.content = "User: " + username + ", Password: " + password
