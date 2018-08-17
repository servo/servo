import json

def main(request, response):
    headers = [("Content-Type", "text/javascript")]

    body = "var header = %s;" % json.dumps(request.headers.get("sec-metadata", ""));

    return headers, body
