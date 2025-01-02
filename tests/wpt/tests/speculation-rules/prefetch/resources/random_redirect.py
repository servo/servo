import secrets
from urllib.parse import urlparse

def main(request, response):
    response.status = 302
    location = request.GET.first(b"location")

    # Add a query param. This is hacky but sufficient.
    location += (b'&' if urlparse(location).query else b'?') + b'random=' + \
        secrets.token_urlsafe(32).encode('utf-8')

    response.headers.set(b"Location", location)
