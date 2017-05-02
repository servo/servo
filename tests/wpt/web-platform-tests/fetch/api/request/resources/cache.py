def main(request, response):
    token = request.GET.first("token", None)
    if "querystate" in request.GET:
        from json import JSONEncoder
        response.headers.set("Content-Type", "text/plain")
        return JSONEncoder().encode(request.server.stash.take(token))
    content = request.GET.first("content", None)
    tag = request.GET.first("tag", None)
    date = request.GET.first("date", None)
    expires = request.GET.first("expires", None)
    vary = request.GET.first("vary", None)
    cc = request.GET.first("cache_control", None)
    redirect = request.GET.first("redirect", None)
    inm = request.headers.get("If-None-Match", None)
    ims = request.headers.get("If-Modified-Since", None)
    pragma = request.headers.get("Pragma", None)
    cache_control = request.headers.get("Cache-Control", None)
    ignore = "ignore" in request.GET

    if tag:
        tag = '"%s"' % tag

    server_state = request.server.stash.take(token)
    if not server_state:
        server_state = []
    state = dict()
    if not ignore:
        if inm:
            state["If-None-Match"] = inm
        if ims:
            state["If-Modified-Since"] = ims
        if pragma:
            state["Pragma"] = pragma
        if cache_control:
            state["Cache-Control"] = cache_control
    server_state.append(state)
    request.server.stash.put(token, server_state)

    if tag:
        response.headers.set("ETag", '%s' % tag)
    elif date:
        response.headers.set("Last-Modified", date)
    if expires:
        response.headers.set("Expires", expires)
    if vary:
        response.headers.set("Vary", vary)
    if cc:
        response.headers.set("Cache-Control", cc)

    # The only-if-cached redirect tests wants CORS to be okay, the other tests
    # are all same-origin anyways and don't care.
    response.headers.set("Access-Control-Allow-Origin", "*");

    if redirect:
        response.headers.set("Location", redirect);
        response.status = (302, "Redirect")
        return ""
    elif ((inm is not None and inm == tag) or
          (ims is not None and ims == date)):
        response.status = (304, "Not Modified")
        return ""
    else:
        response.status = (200, "OK")
        response.headers.set("Content-Type", "text/plain")
        return content
