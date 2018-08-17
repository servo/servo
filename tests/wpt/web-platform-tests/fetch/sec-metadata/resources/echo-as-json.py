import json

def main(request, response):
    headers = [("Content-Type", "application/json"),
               ("Access-Control-Allow-Credentials", "true")]

    if "origin" in request.headers:
        headers.append(("Access-Control-Allow-Origin", request.headers["origin"]))


    body = json.dumps({ "header": request.headers.get("sec-metadata", "") })
    return headers, body
