import os


def main(request, response):
    origin = request.headers.get(b"origin")

    if origin is not None:
        response.headers.set(b"Access-Control-Allow-Origin", origin)
        response.headers.set(b"Access-Control-Allow-Methods", b"GET")
        response.headers.set(b"Access-Control-Allow-Credentials", b"true")

    headers = [
        (b"Content-Type", b"application/webbundle"),
        (b"X-Content-Type-Options", b"nosniff"),
    ]

    cookie = request.cookies.first(b"milk", None)
    if (cookie is not None) and cookie.value == b"1":
        if request.GET.get(b"bundle", None) == b"cross-origin":
            bundle = "./wbn/simple-cross-origin.wbn"
        else:
            bundle = "./wbn/subresource.wbn"
        with open(
            os.path.join(os.path.dirname(__file__), bundle),
            "rb",
        ) as f:
            return (200, headers, f.read())
    else:
        return (400, [], "")
