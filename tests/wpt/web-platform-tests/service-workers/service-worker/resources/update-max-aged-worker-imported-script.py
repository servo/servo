import time

def main(request, response):
    headers = [('Content-Type', 'application/javascript'),
               ('Cache-Control', 'max-age=86400'),
               ('Last-Modified', time.strftime("%a, %d %b %Y %H:%M:%S GMT", time.gmtime()))]

    body = '''
        const importTime = {time:8f};
    '''.format(time=time.time())

    return headers, body
