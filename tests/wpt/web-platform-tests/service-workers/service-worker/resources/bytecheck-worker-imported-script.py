import time

def main(request, response):
    headers = [('Content-Type', 'application/javascript'),
               ('Cache-Control', 'max-age=0')]

    imported_content_type = ''
    if 'imported' in request.GET:
        imported_content_type = request.GET['imported']

    imported_content = 'default'
    if imported_content_type == 'time':
        imported_content = '%f' % time.time()

    body = '''
    // %s
    ''' % (imported_content)

    return headers, body
