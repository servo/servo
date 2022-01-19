import os

# Returns the content of ./wbn/subresource.wbn if the request has an "Accept"
# header including "application/webbundle;v=b1" type, otherwise returns an
# empty body with status code 400.

def main(request, response):
    headers = [
        (b"Content-Type", b"application/webbundle"),
        (b"X-Content-Type-Options", b"nosniff"),
    ]

    accept_values = request.headers.get(b"accept", b"").split(b",")
    if b"application/webbundle;v=b1" in accept_values:
        with open(
            os.path.join(os.path.dirname(__file__), "./wbn/subresource.wbn"),
            "rb",
        ) as f:
            return (200, headers, f.read())
    else:
        return (400, [], "")
