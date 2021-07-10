import re

from wptserve.utils import isomorphic_encode

def main(request, response):
    response.status = 302
    location = re.sub(b'redirect-throw-function.*',
                      b'throw-function.js?secret=1234#ref',
                      isomorphic_encode(request.url))
    response.headers.set(b"Location", location)
