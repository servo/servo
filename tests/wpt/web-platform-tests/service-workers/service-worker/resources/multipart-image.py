# A request handler that serves a multipart image.

import os


BOUNDARY = 'cutHere'


def create_part(path):
    with open(path, 'rb') as f:
        return 'Content-Type: image/png\r\n\r\n' + f.read() + '--%s' % BOUNDARY


def main(request, response):
    content_type = 'multipart/x-mixed-replace; boundary=%s' % BOUNDARY
    headers = [('Content-Type', content_type)]
    if 'approvecors' in request.GET:
        headers.append(('Access-Control-Allow-Origin', '*'))

    image_path = os.path.join(request.doc_root, 'images')
    body = create_part(os.path.join(image_path, 'red.png'))
    body = body + create_part(os.path.join(image_path, 'red-16x16.png'))
    return headers, body
