import os

from wptserve.utils import isomorphic_encode

filename = os.path.basename(isomorphic_encode(__file__))

def main(request, response):
    if request.method == u'POST':
        return 302, [(b'Location', b'./%s?redirect' % filename)], b''

    return [(b'Content-Type', b'text/plain')], request.request_path
