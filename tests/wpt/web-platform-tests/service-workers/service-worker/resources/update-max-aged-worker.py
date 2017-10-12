import time
import json

def main(request, response):
    headers = [('Content-Type', 'application/javascript'),
               ('Cache-Control', 'max-age=86400'),
               ('Last-Modified', time.strftime("%a, %d %b %Y %H:%M:%S GMT", time.gmtime()))]

    test = request.GET['test']

    body = '''
        const mainTime = {time:8f};
        const testName = {test};
        importScripts('update-max-aged-worker-imported-script.py');

        addEventListener('message', event => {{
            event.source.postMessage({{
                mainTime,
                importTime,
                test: {test}
            }});
        }});
    '''.format(
        time=time.time(),
        test=json.dumps(test)
    )

    return headers, body
