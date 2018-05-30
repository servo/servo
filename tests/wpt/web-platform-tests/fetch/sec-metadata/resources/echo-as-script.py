import json

def main(request, response):
    headers = [("Content-Type", "text/javascript")]

    body = "var header = %s;" % json.dumps(request.headers["sec-metadata"]);

    return headers, body
