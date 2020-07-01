def main(request, response):
    if b'Status' in request.GET:
        status = int(request.GET[b"Status"])
    else:
        status = 302

    headers = []

    url = request.GET[b'Redirect']
    headers.append((b"Location", url))

    if b"ACAOrigin" in request.GET:
        for item in request.GET[b"ACAOrigin"].split(b","):
            headers.append((b"Access-Control-Allow-Origin", item))

    for suffix in [b"Headers", b"Methods", b"Credentials"]:
        query = b"ACA%s" % suffix
        header = b"Access-Control-Allow-%s" % suffix
        if query in request.GET:
            headers.append((header, request.GET[query]))

    if b"ACEHeaders" in request.GET:
        headers.append((b"Access-Control-Expose-Headers", request.GET[b"ACEHeaders"]))

    return status, headers, b""
