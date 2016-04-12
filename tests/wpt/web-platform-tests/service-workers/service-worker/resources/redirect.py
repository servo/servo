def main(request, response):
    if 'Status' in request.GET:
        status = int(request.GET["Status"])
    else:
        status = 302

    headers = []

    url = request.GET['Redirect']
    headers.append(("Location", url))

    if "ACAOrigin" in request.GET:
        for item in request.GET["ACAOrigin"].split(","):
            headers.append(("Access-Control-Allow-Origin", item))

    for suffix in ["Headers", "Methods", "Credentials"]:
        query = "ACA%s" % suffix
        header = "Access-Control-Allow-%s" % suffix
        if query in request.GET:
            headers.append((header, request.GET[query]))

    if "ACEHeaders" in request.GET:
        headers.append(("Access-Control-Expose-Headers", request.GET["ACEHeaders"]))

    return status, headers, ""
