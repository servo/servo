def main(request, response):
    response.headers.set(b"Cache-Control", b"no-store")
    response.headers.set(b"Access-Control-Allow-Origin", request.headers.get(b"origin"))
    response.headers.set(b"Access-Control-Allow-Credentials", b"true")
    uid = request.GET.first(b"uid", None)

    if request.method == u"OPTIONS":
        response.headers.set(b"Access-Control-Allow-Methods", b"PUT")
    else:
        username = request.auth.username
        password = request.auth.password
        if (not username) or (username != uid):
            response.headers.set(b"WWW-Authenticate", b"Basic realm='Test Realm/Cross Origin'")
            response.status = 401
            response.content = b"Authentication cancelled"
        else:
            response.content = b"User: " + username + b", Password: " + password
