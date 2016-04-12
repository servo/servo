import json

def main(request, response):
    data = {key:request.headers[key] for key,value in request.headers.iteritems()}

    return [("Content-Type", "application/json")], json.dumps(data)
