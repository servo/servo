# A request handler that serves a multipart image.

import os


BOUNDARY = b'cutHere'


def create_part(path):
    with open(path, u'rb') as f:
        return b'Content-Type: image/png\r\n\r\n' + f.read() + b'--%s' % BOUNDARY


def main(request, response):
    content_type = b'multipart/x-mixed-replace; boundary=%s' % BOUNDARY
    headers = [(b'Content-Type', content_type)]
    if b'approvecors' in request.GET:
        headers.append((b'Access-Control-Allow-Origin', b'*'))

    image_path = os.path.join(request.doc_root, u'images')
    body = create_part(os.path.join(image_path, u'red.png'))
    body = body + create_part(os.path.join(image_path, u'red-16x16.png'))
    return headers, body
