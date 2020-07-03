import time

def main(request, response):
    headers = [(b'Content-Type', b'application/javascript'),
               (b'Cache-Control', b'max-age=0')]

    imported_content_type = b''
    if b'imported' in request.GET:
        imported_content_type = request.GET[b'imported']

    imported_content = b'default'
    if imported_content_type == b'time':
        imported_content = b'%f' % time.time()

    body = b'''
    // %s
    ''' % (imported_content)

    return headers, body
