import time

def main(request, response):
    headers = [(b'Content-Type', b'application/javascript'),
               (b'Cache-Control', b'max-age=0')]

    main_content_type = b''
    if b'main' in request.GET:
        main_content_type = request.GET[b'main']

    main_content = b'default'
    if main_content_type == b'time':
        main_content = b'%f' % time.time()

    imported_request_path = b''
    if b'path' in request.GET:
        imported_request_path = request.GET[b'path']

    imported_request_type = b''
    if b'imported' in request.GET:
        imported_request_type = request.GET[b'imported']

    imported_request = b''
    if imported_request_type == b'time':
        imported_request = b'?imported=time'

    if b'type' in request.GET and request.GET[b'type'] == b'module':
        body = b'''
        // %s
        import '%sbytecheck-worker-imported-script.py%s';
        ''' % (main_content, imported_request_path, imported_request)
    else:
        body = b'''
        // %s
        importScripts('%sbytecheck-worker-imported-script.py%s');
        ''' % (main_content, imported_request_path, imported_request)

    return headers, body
