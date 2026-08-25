import os

from wptserve.utils import isomorphic_encode

def main(request, response):
    origin = request.GET.first(b'origin')
    if origin:
        response.headers.set(b'Access-Control-Allow-Origin', origin)

    tao = request.GET.first(b'tao')

    if tao == b'wildcard':
    # wildcard, pass
        response.headers.set(b'Timing-Allow-Origin', b'*')
    elif tao == b'null':
    # null, fail
        response.headers.set(b'Timing-Allow-Origin', b'null')
    elif tao == b'origin':
    # case-sensitive match for origin, pass
        response.headers.set(b'Timing-Allow-Origin', origin)
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
    response.headers.set(b"Cache-Control", b"no-cache, must-revalidate");
    image_path = os.path.join(os.path.dirname(isomorphic_encode(__file__)), b"square100.png");
    response.content = open(image_path, mode=u'rb').read();
