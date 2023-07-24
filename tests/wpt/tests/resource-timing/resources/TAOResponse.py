import os

def main(request, response):
    if b'origin' in request.headers:
      origin = request.headers[b'origin']
      response.headers.set(b'Access-Control-Allow-Origin', origin)

    tao = request.GET.first(b'tao')
    img = request.GET.first(b'img') if b'img' in request.GET else None

    if tao == b'zero':
    # zero TAO value, fail
        pass
    elif tao == b'wildcard':
    # wildcard, pass
        response.headers.set(b'Timing-Allow-Origin', b'*')
    elif tao == b'null':
    # null, fail unless it's an opaque origin
        response.headers.set(b'Timing-Allow-Origin', b'null')
    elif tao == b'Null':
    # case-insensitive null, fail
        response.headers.set(b'Timing-Allow-Origin', b'Null')
    elif tao == b'origin':
    # case-sensitive match for origin, pass
        response.headers.set(b'Timing-Allow-Origin', origin)
    elif tao.startswith(b'origin_port'):
    # case-sensitive match for origin and port, pass
        origin_parts = origin.split(b':')
        host = origin_parts[0] + b':' + origin_parts[1]
        port = tao.split(b'origin_port_')[1]
        response.headers.set(b'Timing-Allow-Origin', host + b':' + port)
    elif tao == b'space':
    # space separated list of origin and wildcard, fail
        response.headers.set(b'Timing-Allow-Origin', (origin + b' *'))
    elif tao == b'multi':
    # more than one TAO values, separated by comma, pass
        response.headers.set(b'Timing-Allow-Origin', origin)
        response.headers.append(b'Timing-Allow-Origin', b'*')
    elif tao == b'multi_wildcard':
    # multiple wildcards, separated by comma, pass
        response.headers.set(b'Timing-Allow-Origin', b'*')
        response.headers.append(b'Timing-Allow-Origin', b'*')
    elif tao == b'match_origin':
    # contains a match of origin, separated by comma, pass
        response.headers.set(b'Timing-Allow-Origin', origin)
        response.headers.append(b'Timing-Allow-Origin', b"fake")
    elif tao == b'match_wildcard':
    # contains a wildcard, separated by comma, pass
        response.headers.set(b'Timing-Allow-Origin', b"fake")
        response.headers.append(b'Timing-Allow-Origin', b'*')
    elif tao == b'uppercase':
    # non-case-sensitive match for origin, fail
        response.headers.set(b'Timing-Allow-Origin', origin.upper())
    else:
        pass
    response.status = 200
    if img:
      response.headers.set(b"Content-Type", b"image/png")
      with open(request.doc_root + "/resource-timing/resources/blue.png", "rb") as f:
        response.content = f.read()
        f.close()
    else:
      response.headers.set(b"Content-Type", b"text/plain")
      response.content = "TEST"
