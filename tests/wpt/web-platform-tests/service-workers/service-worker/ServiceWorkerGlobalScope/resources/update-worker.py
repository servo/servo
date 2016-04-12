import os
import time

def main(request, response):
    # update() does not bypass cache so set the max-age to 0 such that update()
    # can find a new version in the network.
    headers = [('Cache-Control', 'max-age: 0'),
               ('Content-Type', 'application/javascript')]
    with open(os.path.join(os.path.dirname(__file__),
                           'update-worker.js'), 'r') as file:
        script = file.read()
    # Return a different script for each access.
    return headers, '// %s\n%s' % (time.time(), script)

