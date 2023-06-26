import time
import json

from wptserve.utils import isomorphic_decode, isomorphic_encode

def main(request, response):
    headers = [(b'Content-Type', b'application/javascript'),
               (b'Cache-Control', b'max-age=86400'),
               (b'Last-Modified', isomorphic_encode(time.strftime(u"%a, %d %b %Y %H:%M:%S GMT", time.gmtime())))]

    test = request.GET[b'test']

    body = u'''
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
        test=json.dumps(isomorphic_decode(test))
    )

    return headers, body
