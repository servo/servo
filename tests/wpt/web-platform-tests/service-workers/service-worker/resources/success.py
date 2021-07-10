def main(request, response):
    headers = []

    if b"ACAOrigin" in request.GET:
        for item in request.GET[b"ACAOrigin"].split(b","):
            headers.append((b"Access-Control-Allow-Origin", item))

    return headers, b"{ \"result\": \"success\" }"
