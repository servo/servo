import os
import time

def main(request, response):
    if 'server_slow' in request.url_parts.query:
        time.sleep(0.2)
    if 'server_no_content' in request.url_parts.query:
        return 204, [(b'Content-Type', b'text/plain')], u'Network with %s request' % request.method
    if 'server_not_found' in request.url_parts.query:
        return 404, [(b'Content-Type', b'text/plain')], u'Not Found'
    return 200, [(b'Content-Type', b'text/plain')], u'Network with %s request' % request.method
