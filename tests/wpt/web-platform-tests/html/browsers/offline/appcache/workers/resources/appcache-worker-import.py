script = b'info = \'Set by the %s script\';'

def main(request, response):
    type = request.GET[b'type']
    if request.GET[b'type'] == b'fallingback':
        return 404, [(b'Content-Type', b'text/plain')], u"Page not found"
    return [(b'Content-Type', b'text/javascript')], script % type
