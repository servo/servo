import os
import time

def main(request, response):
    if 'server_slow' in request.url_parts.query:
        time.sleep(0.2)
    return 200, [(b'Content-Type', b'text/plain')], u'Network with %s request' % request.method
