import urllib
import os.path

def main(request, response):
    coop = request.GET.first('coop')
    if coop:
        response.headers.set('Cross-Origin-Opener-Policy', urllib.unquote(coop))

    path = os.path.join(os.path.dirname(__file__), request.GET.first('path'))
    response.content = open(path, mode='rb').read()
