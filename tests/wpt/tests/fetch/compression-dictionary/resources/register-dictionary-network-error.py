def main(request, response):
    # https://fetch.spec.whatwg.org/#http-redirect-fetch
    # If locationURL’s scheme is not an HTTP(S) scheme, then return a network error.
    response.status = 302
    response.headers.set(b"Location", "about:blank")
