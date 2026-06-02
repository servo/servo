import re

def main(request, response):
    headers = [(b"Content-Type", b"text/javascript")]
    test = request.GET.first(b'test')
    assert(re.match(b'^[a-zA-Z0-9_]+$', test))

    if test.find(b'_load') >= 0:
      status = 200
      content = b'"use strict"; %s.executed = true;' % test
    else:
      status = 404
      content = b'"use strict"; %s.test.step(function() { assert_unreached("404 script should not be executed"); });' % test

    return status, headers, content
