import time

def main(request, response):
    headers = [('Content-Type', 'application/javascript'),
               ('Cache-Control', 'max-age=0')]

    main_content_type = ''
    if 'main' in request.GET:
        main_content_type = request.GET['main']

    main_content = 'default'
    if main_content_type == 'time':
        main_content = '%f' % time.time()

    imported_request_path = ''
    if 'path' in request.GET:
        imported_request_path = request.GET['path']

    imported_request_type = ''
    if 'imported' in request.GET:
        imported_request_type = request.GET['imported']

    imported_request = ''
    if imported_request_type == 'time':
        imported_request = '?imported=time';

    body = '''
    // %s
    importScripts('%sbytecheck-worker-imported-script.py%s');
    ''' % (main_content, imported_request_path, imported_request)

    return headers, body
