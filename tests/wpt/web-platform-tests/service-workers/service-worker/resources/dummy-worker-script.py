def main(request, response):
    headers = []

    if "ACAOrigin" in request.GET:
        for item in request.GET["ACAOrigin"].split(","):
            headers.append(("Access-Control-Allow-Origin", item))

    return headers, "postMessage('dummy-worker-script loaded');"

