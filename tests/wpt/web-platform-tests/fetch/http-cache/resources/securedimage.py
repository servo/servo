# -*- coding: utf-8 -

def main(request, response):
    image_url = str.replace(request.url, "fetch/http-cache/resources/securedimage.py", "images/green.png")

    if "authorization" not in request.headers:
        response.status = 401
        response.headers.set("WWW-Authenticate", "Basic")
        return
    else:
        auth = request.headers.get("Authorization")
        if auth != "Basic dGVzdHVzZXI6dGVzdHBhc3M=":
            response.set_error(403, "Invalid username or password - " + auth)
            return

    response.status = 301
    response.headers.set("Location", image_url)
