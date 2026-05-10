import re
import time

def main(request, response):
    test = request.GET.first(b'test')
    assert(re.match(b'^[a-zA-Z0-9_]+$', test))

    delay = int(request.GET.first(b'delay', 0))
    time.sleep(delay);

    if test.find(b'_svg') >= 0:
        mime_type = b'image/svg+xml'
        content = b'<?xml version="1.0" encoding="UTF-8"?><svg xmlns="http://www.w3.org/2000/svg"></svg>'
    elif test.find(b'_script') >= 0:
        mime_type = b'text/javascript'
        content = b'// comment'
    else:
        mime_type = b'text/plain'
        content = b'text'

    headers = [(b'Content-Type', mime_type)]

    if test.find(b'_load') >= 0:
      status = 200
    else:
      status = 404

    return status, headers, content
