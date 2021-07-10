import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    data = {isomorphic_decode(key):isomorphic_decode(request.headers[key]) for key, value in request.headers.items()}

    return [(b"Content-Type", b"application/json")], json.dumps(data)
