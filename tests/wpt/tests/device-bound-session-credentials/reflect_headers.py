import json

from wptserve.utils import isomorphic_decode

# Returns the request headers as JSON, so we can test if a header is
# included or excluded.
def main(request, response):
    normalized = dict()

    for key, values in dict(request.headers).items():
        new_values = [isomorphic_decode(value) for value in values]
        normalized[isomorphic_decode(key.lower())] = new_values

    headers = []
    if request.headers.get(b"origin") != None:
      headers = [(b"Access-Control-Allow-Origin",request.headers.get(b"origin")),
                 (b"Access-Control-Allow-Credentials", b"true")]

    return (200, headers, json.dumps(normalized))
