import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    if b"origin" in request.GET:
        response.headers.set(b"Access-Control-Allow-Origin", request.GET[b"origin"])
    elif b"origins" in request.GET:
        for origin in request.GET[b"origins"].split(b','):
            response.headers.set(b"Access-Control-Allow-Origin", request.GET[b"origin"])

    if b"headers" in request.GET:
        response.headers.set(b"Access-Control-Allow-Headers", request.GET[b"headers"])
    if b"methods" in request.GET:
        response.headers.set(b"Access-Control-Allow-Methods", request.GET[b"methods"])

    headers = dict(request.headers)

    for header in headers:
        headers[header] = headers[header][0]

    str_headers = {}
    for key, val in headers.items():
        str_headers[isomorphic_decode(key)] = isomorphic_decode(val)

    return json.dumps(str_headers)
