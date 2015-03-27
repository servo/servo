import json

def main(request, response):
    origin = request.GET.first("origin", request.headers.get('origin'))

    if "check" in request.GET:
        token = request.GET.first("token")
        value = request.server.stash.take(token)
        if value is not None:
            if request.GET.first("check", None) == "keep":
                request.server.stash.put(token, value)
            body = "1"
        else:
            body = "0"
        return [("Content-Type", "text/plain")], body


    if origin != 'none':
        response.headers.set("Access-Control-Allow-Origin", origin)
    if 'origin2' in request.GET:
        response.headers.append("Access-Control-Allow-Origin", request.GET.first('origin2'))

    #Preflight
    if 'headers' in request.GET:
        response.headers.set("Access-Control-Allow-Headers", request.GET.first('headers'))
    if 'credentials' in request.GET:
        response.headers.set("Access-Control-Allow-Credentials", request.GET.first('credentials'))
    if 'methods' in request.GET:
        response.headers.set("Access-Control-Allow-Methods", request.GET.first('methods'))

    code = request.GET.first('code', None)
    if request.method == 'OPTIONS':
        #Override the response code if we're in a preflight and it's asked
        if 'preflight' in request.GET:
            code = int(request.GET.first('preflight'))

        #Log that the preflight actually happened if we have an ident
        if 'token' in request.GET:
            request.server.stash.put(request.GET['token'])

    if 'location' in request.GET:
        if code is None:
            code = 302

        if code >= 300 and code < 400:
            response.headers.set("Location", request.GET.first('location'))

    headers = {}
    for name, values in request.headers.iteritems():
        if len(values) == 1:
            headers[name] = values[0]
        else:
            #I have no idea, really
            headers[name] = values

    headers['get_value'] = request.GET.first('get_value', '')

    body = json.dumps(headers)

    if code:
        return (code, "StatusText"), [], body
    else:
        return body

