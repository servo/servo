import json
from cookies.resources import helpers

from wptserve.utils import isomorphic_decode

def main(request, response):
    headers = helpers.setNoCacheAndCORSHeaders(request, response)
    headers[0] = (b"Content-Type", b"text/javascript")
    cookies = helpers.readCookies(request)
    decoded_cookies = {isomorphic_decode(key): isomorphic_decode(val) for key, val in cookies.items()}
    return headers, 'self._cookies = [{}];\n'.format(
        ', '.join(['"{}"'.format(name) for name in decoded_cookies.keys()]))
