import json
from cookies.resources.helpers import makeCookieHeader, readCookies, setNoCacheAndCORSHeaders

from wptserve.utils import isomorphic_decode

def main(request, response):
    headers = setNoCacheAndCORSHeaders(request, response)
    cookies = readCookies(request)
    decoded_cookies = {isomorphic_decode(key): isomorphic_decode(val) for key, val in cookies.items()}
    return headers, json.dumps(decoded_cookies)