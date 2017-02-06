import time

def main(request, response):
    headers = [('Content-Type', 'application/javascript'),
               ('Cache-Control', 'max-age=86400'),
               ('Last-Modified', time.strftime("%a, %d %b %Y %H:%M:%S GMT", time.gmtime()))]

    test = '';
    if 'Test' in request.GET:
      test = request.GET['Test'];

    revalidate = request.headers.has_key('if-modified-since');

    body = '''
    importScripts('update-max-aged-worker-imported-script.py?Test=%s');

    self.addEventListener('message', function(e) {
        e.data.port.postMessage({
            from: "main",
            type: "%s",
            value: %s
        });
    });
    ''' % (test, 'revalidate' if revalidate else 'normal', time.time())

    return headers, body
