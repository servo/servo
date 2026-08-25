import time

from wptserve.utils import isomorphic_encode

def main(request, response):
    headers = [(b'Content-Type', b'application/javascript'),
               (b'Cache-Control', b'max-age=86400'),
               (b'Last-Modified', isomorphic_encode(time.strftime(u"%a, %d %b %Y %H:%M:%S GMT", time.gmtime())))]

    body = u'''
        const importTime = {time:8f};
    '''.format(time=time.time())

    return headers, body
