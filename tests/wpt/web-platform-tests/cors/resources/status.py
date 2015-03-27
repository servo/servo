def main(request, response):
    response.headers.set("Access-Control-Allow-Origin", request.headers.get("origin") )
    response.headers.set("Access-Control-Expose-Headers", "X-Request-Method")

    if request.method == 'OPTIONS':
        response.headers.set("Access-Control-Allow-Methods",  "GET, CHICKEN, HEAD, POST, PUT")

    if 'headers' in request.GET:
        response.headers.set("Access-Control-Allow-Headers",  request.GET.first('headers'))

    response.headers.set("X-Request-Method", request.method)

    response.headers.set("X-A-C-Request-Method", request.headers.get("Access-Control-Request-Method", ""));


    #This should reasonably work for most response codes.
    try:
        code = int(request.GET.first("code", 200))
    except ValueError:
        code = 200

    text = request.GET.first("text", "OMG")

    if request.method == "OPTIONS" and "preflight" in request.GET:
        try:
            code = int(request.GET.first('preflight'))
        except KeyError, ValueError:
            pass

    status = code, text

    if "type" in request.GET:
        response.headers.set("Content-Type", request.GET.first('type'))

    body = request.GET.first('content', "")

    return status, [], body
