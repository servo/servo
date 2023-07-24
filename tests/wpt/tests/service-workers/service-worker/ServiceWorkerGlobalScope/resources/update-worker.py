import os
import time

from wptserve.utils import isomorphic_decode

def main(request, response):
    # update() does not bypass cache so set the max-age to 0 such that update()
    # can find a new version in the network.
    headers = [(b'Cache-Control', b'max-age: 0'),
               (b'Content-Type', b'application/javascript')]
    with open(os.path.join(os.path.dirname(isomorphic_decode(__file__)),
                           u'update-worker.js'), u'r') as file:
        script = file.read()
    # Return a different script for each access.
    return headers, u'// %s\n%s' % (time.time(), script)

