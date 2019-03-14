import os

def main(request, response):
    origin = request.GET.first('origin', '');
    if origin:
        response.headers.set('Access-Control-Allow-Origin', origin)

    tao = request.GET.first('tao')

    if tao == 'wildcard':
    # wildcard, pass
        response.headers.set('Timing-Allow-Origin', '*')
    elif tao == 'null':
    # null, fail
        response.headers.set('Timing-Allow-Origin', 'null')
    elif tao == 'origin':
    # case-sensitive match for origin, pass
        response.headers.set('Timing-Allow-Origin', origin)
    elif tao == 'space':
    # space separated list of origin and wildcard, fail
        response.headers.set('Timing-Allow-Origin', (origin + ' *'))
    elif tao == 'multi':
    # more than one TAO values, separated by comma, pass
        response.headers.set('Timing-Allow-Origin', origin)
        response.headers.append('Timing-Allow-Origin', '*')
    elif tao == 'multi_wildcard':
    # multiple wildcards, separated by comma, pass
        response.headers.set('Timing-Allow-Origin', '*')
        response.headers.append('Timing-Allow-Origin', '*')
    elif tao == 'match_origin':
    # contains a match of origin, separated by comma, pass
        response.headers.set('Timing-Allow-Origin', origin)
        response.headers.append('Timing-Allow-Origin', "fake")
    elif tao == 'match_wildcard':
    # contains a wildcard, separated by comma, pass
        response.headers.set('Timing-Allow-Origin', "fake")
        response.headers.append('Timing-Allow-Origin', '*')
    elif tao == 'uppercase':
    # non-case-sensitive match for origin, fail
        response.headers.set('Timing-Allow-Origin', origin.upper())
    else:
        pass
    response.headers.set("Cache-Control", "no-cache, must-revalidate");
    image_path = os.path.join(os.path.dirname(__file__), "square20.png");
    response.content = open(image_path, mode='rb').read();
