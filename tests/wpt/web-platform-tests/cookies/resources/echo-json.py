import json

def main(request, response):
    headers = [("Content-Type", "application/json"),
               ("Access-Control-Allow-Credentials", "true")]

    if "origin" in request.headers:
        headers.append(("Access-Control-Allow-Origin", request.headers["origin"]))

    values = []
    for key in request.cookies:
        for value in request.cookies.get_list(key):
            values.append("\"%s\": \"%s\"" % (key, value))
    body = "{ %s }" % ",".join(values)
    return headers, body
