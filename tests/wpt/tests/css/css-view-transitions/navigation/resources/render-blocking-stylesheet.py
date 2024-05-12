import time


# This handler blocks a GET request for the given key until a matching POST is
# made with the same key. This allows a test to load a resource and manually
# control when the response is received.
def main(request, response):
    key = request.GET.first(b'key')

    if request.method == 'POST':
        # Received result data from target page
        request.server.stash.put(key, 'doResponse')
        return 'done'
    else:
        poll_delay_sec = 0.1

        # Wait until the caller POSTs before responding.
        while request.server.stash.take(key) is None:
            time.sleep(poll_delay_sec)

        status = 200
        headers = [('Content-Type', 'text/css')]
        body = ''
        return (status, headers, body)
