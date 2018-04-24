def main(request, response):
    origin = request.headers['origin']
    response.headers.set('Access-Control-Allow-Origin', origin)

    tao = request.GET.first('tao')

    if tao == 'zero':
    # zero TAO value, fail
        pass
    elif tao == 'wildcard':
    # wildcard, pass
        response.headers.set('Timing-Allow-Origin', '*')
    elif tao == 'null':
    # null, fail
        response.headers.set('Timing-Allow-Origin', 'null')
    elif tao == 'origin':
    # case-sensitive match for origin, pass
        response.headers.set('Timing-Allow-Origin', origin)
    elif tao == 'space':
    # space seperated list of origin and wildcard, fail
        response.headers.set('Timing-Allow-Origin', (origin + ' *'))
    elif tao == 'multi':
    # more than one TAO values, seperated by comma, pass
        response.headers.set('Timing-Allow-Origin', origin)
        response.headers.append('Timing-Allow-Origin', '*')
    elif tao == 'match_origin':
    # contains a match of origin, seperated by comma, pass
        response.headers.set('Timing-Allow-Origin', origin)
        response.headers.append('Timing-Allow-Origin', "fake")
    elif tao == 'match_wildcard':
    # contains a wildcard, seperated by comma, pass
        response.headers.set('Timing-Allow-Origin', "fake")
        response.headers.append('Timing-Allow-Origin', '*')
    elif tao == 'uppercase':
    # non-case-sensitive match for origin, fail
        response.headers.set('Timing-Allow-Origin', origin.upper())
    else:
        pass
