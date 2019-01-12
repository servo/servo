import json

def main(request, response):
    headers = [("Content-Type", "application/json"),
               ("Access-Control-Allow-Credentials", "true")]

    if "origin" in request.headers:
        headers.append(("Access-Control-Allow-Origin", request.headers["origin"]))


    body = json.dumps({
        "dest": request.headers.get("sec-fetch-dest", ""),
        "mode": request.headers.get("sec-fetch-mode", ""),
        "site": request.headers.get("sec-fetch-site", ""),
        "user": request.headers.get("sec-fetch-user", ""),
        })
    return headers, body
