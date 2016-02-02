def main(request, response):
    headers = [("Content-Type", "text/plain")]
    stashed_data = {'control_request_headers': "", 'preflight': "0", 'preflight_referrer': ""}

    if "origin" in request.GET:
        for origin in request.GET['origin'].split(", "):
            headers.append(("Access-Control-Allow-Origin", origin))
    else:
        headers.append(("Access-Control-Allow-Origin", "*"))

    if request.method == "OPTIONS":
        if not "Access-Control-Request-Method" in request.headers:
            response.set_error(400, "No Access-Control-Request-Method header")
            return "ERROR: No access-control-request-method in preflight!"

        if "control_request_headers" in request.GET:
            stashed_data['control_request_headers'] = request.headers.get("Access-Control-Request-Headers", "")

        if "max_age" in request.GET:
            headers.append(("Access-Control-Max-Age", request.GET['max_age']))

        if "allow_headers" in request.GET:
            headers.append(("Access-Control-Allow-Headers", request.GET['allow_headers']))

        if "allow_methods" in request.GET:
            headers.append(("Access-Control-Allow-Methods", request.GET['allow_methods']))

        preflight_status = 200
        if "preflight_status" in request.GET:
            preflight_status = int(request.GET.first("preflight_status"))

        stashed_data['preflight'] = "1"
        stashed_data['preflight_referrer'] = request.headers.get("Referer", "")
        request.server.stash.put(request.GET.first("token"), stashed_data)

        return preflight_status, headers, ""

    token = None
    if "token" in request.GET:
        token = request.GET.first("token")
        data = request.server.stash.take(token)
        if data:
            stashed_data = data

    #use x-* headers for returning value to bodyless responses
    headers.append(("Access-Control-Expose-Headers", "x-did-preflight, x-control-request-headers, x-referrer, x-preflight-referrer, x-origin"))
    headers.append(("x-did-preflight", stashed_data['preflight']))
    headers.append(("x-control-request-headers", stashed_data['control_request_headers']))
    headers.append(("x-preflight-referrer", stashed_data['preflight_referrer']))
    headers.append(("x-referrer", request.headers.get("Referer", "") ))
    headers.append(("x-origin", request.headers.get("Origin", "") ))

    if token:
      request.server.stash.put(token, stashed_data)

    return headers, ""
