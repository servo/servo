import os

filename = os.path.basename(__file__)

def main(request, response):
    if request.method == 'POST':
        return 302, [('Location', './%s?redirect' % filename)], ''

    return [('Content-Type', 'text/plain')], request.request_path
