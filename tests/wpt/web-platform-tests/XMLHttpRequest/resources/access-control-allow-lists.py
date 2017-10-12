import json

def main(request, response):
    if "origin" in request.GET:
        response.headers.set("Access-Control-Allow-Origin", request.GET["origin"])
    elif "origins" in request.GET:
        for origin in request.GET["origins"].split(','):
            response.headers.set("Access-Control-Allow-Origin", request.GET["origin"])

    if "headers" in request.GET:
        response.headers.set("Access-Control-Allow-Headers", request.GET["headers"])
    if "methods" in request.GET:
        response.headers.set("Access-Control-Allow-Methods", request.GET["methods"])

    headers = dict(request.headers)

    for header in headers:
        headers[header] = headers[header][0]

    return json.dumps(headers)
