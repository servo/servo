def main(request, response):
    headers = []
    request_headers = []
    if "headers" in request.GET:
        checked_headers = request.GET.first("headers").split("|")
        for header in checked_headers:
          if header in request.headers:
              headers.append(("x-request-" + header, request.headers.get(header, "") ))

    if "cors" in request.GET:
        if "Origin" in request.headers:
            headers.append(("Access-Control-Allow-Origin", request.headers.get("Origin", "")))
        else:
            headers.append(("Access-Control-Allow-Origin", "*"))
        headers.append(("Access-Control-Allow-Credentials", "true"))
        headers.append(("Access-Control-Allow-Methods", "GET, POST, HEAD"))
        exposed_headers = ["x-request-" + header for header in checked_headers]
        headers.append(("Access-Control-Expose-Headers", ", ".join(exposed_headers)))
        headers.append(("Access-Control-Allow-Headers", ", ".join(request.headers)))

    headers.append(("content-type", "text/plain"))
    return headers, ""
