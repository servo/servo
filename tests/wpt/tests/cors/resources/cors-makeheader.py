import json

from wptserve.utils import isomorphic_decode

def main(request, response):
    origin = request.GET.first(b"origin", request.headers.get(b'origin') or b'none')

    if b"check" in request.GET:
        token = request.GET.first(b"token")
        value = request.server.stash.take(token)
        if value is not None:
            if request.GET.first(b"check", None) == b"keep":
                request.server.stash.put(token, value)
            body = u"1"
        else:
            body = u"0"
        return [(b"Content-Type", b"text/plain")], body


    if origin != b'none':
        response.headers.set(b"Access-Control-Allow-Origin", origin)
    if b'origin2' in request.GET:
        response.headers.append(b"Access-Control-Allow-Origin", request.GET.first(b'origin2'))

    #Preflight
    if b'headers' in request.GET:
        response.headers.set(b"Access-Control-Allow-Headers", request.GET.first(b'headers'))
    if b'credentials' in request.GET:
        response.headers.set(b"Access-Control-Allow-Credentials", request.GET.first(b'credentials'))
    if b'methods' in request.GET:
        response.headers.set(b"Access-Control-Allow-Methods", request.GET.first(b'methods'))

    code_raw = request.GET.first(b'code', None)
    if code_raw:
        code = int(code_raw)
    else:
        code = None
    if request.method == u'OPTIONS':
        #Override the response code if we're in a preflight and it's asked
        if b'preflight' in request.GET:
            code = int(request.GET.first(b'preflight'))

        #Log that the preflight actually happened if we have an ident
        if b'token' in request.GET:
            request.server.stash.put(request.GET[b'token'], True)

    if b'location' in request.GET:
        if code is None:
            code = 302

        if code >= 300 and code < 400:
            response.headers.set(b"Location", request.GET.first(b'location'))

    headers = {}
    for name, values in request.headers.items():
        if len(values) == 1:
            headers[isomorphic_decode(name)] = isomorphic_decode(values[0])
        else:
            #I have no idea, really
            headers[name] = values

    headers[u'get_value'] = isomorphic_decode(request.GET.first(b'get_value', b''))

    body = json.dumps(headers)

    if code:
        return (code, b"StatusText"), [], body
    else:
        return body
