# -*- coding: utf-8 -

def main(request, response):
    image_url = str.replace(request.url, "http/resources/securedimage.py", "images/green.png")

    if "authorization" not in request.headers:
        response.status = 401
        response.headers.set("WWW-Authenticate", "Basic")
    else:
        auth = request.headers.get("Authorization")
        if auth != "Basic dGVzdHVzZXI6dGVzdHBhc3M=":
            response.set_error(403, "Invalid username or password - " + auth)

    response.status = 301
    response.headers.set("Location", image_url)
