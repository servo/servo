import os


def main(request, response):
    origin = request.headers.get(b"origin")
    if origin is not None:
        response.headers.set(b"Access-Control-Allow-Origin", origin)

    headers = [
        (b"Content-Type", b"application/webbundle"),
        (b"X-Content-Type-Options", b"nosniff"),
    ]

    if request.headers.get(b"sec-fetch-dest", None) == b"webbundle":
        with open(
            os.path.join(os.path.dirname(__file__), "./wbn/subresource.wbn"),
            "rb",
        ) as f:
            return (200, headers, f.read())
    else:
        return (400, [], "")
