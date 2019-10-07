import json

def main(request, response):
    headers = [("Content-Type", "text/javascript")]

    body = "var header = %s;" % json.dumps({
        "dest": request.headers.get("sec-fetch-dest", ""),
        "mode": request.headers.get("sec-fetch-mode", ""),
        "site": request.headers.get("sec-fetch-site", ""),
        "user": request.headers.get("sec-fetch-user", ""),
        });

    return headers, body
