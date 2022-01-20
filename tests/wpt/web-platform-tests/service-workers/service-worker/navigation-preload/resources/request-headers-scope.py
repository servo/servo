import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    normalized = dict()

    for key, values in dict(request.headers).items():
        values = [isomorphic_decode(value) for value in values]
        normalized[isomorphic_decode(key.upper())] = values

    response.headers.append(b"Content-Type", b"text/html")

    return json.dumps(normalized)
