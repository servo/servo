import json

def main(request, response):
    normalized = dict()

    for key, values in dict(request.headers).iteritems():
        normalized[key.upper()] = values

    return json.dumps(normalized)
