import time

def main(request, response):
    headers = [('Cache-Control', 'max-age=86400'),
               ('Content-Type', 'application/javascript'),
               ('Last-Modified', time.strftime("%a, %d %b %Y %H:%M:%S GMT",
                                               time.gmtime()))]


    revalidate = request.headers.has_key('if-modified-since');

    body = '''
    self.addEventListener('message', function(e) {
        e.data.port.postMessage({
            from: "imported",
            type: "%s",
            value: %s
        });
    });
    ''' % ('revalidate' if revalidate else 'normal', time.time())

    return headers, body
