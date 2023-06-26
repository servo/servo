import re

def main(request, response):
    headers = [(b"Content-Type", b"text/javascript")]
    test = request.GET.first(b'test')
    assert(re.match(b'^[a-zA-Z0-9_]+$', test))

    status = 200
    if test.find(b'_load') >= 0:
      content = b'import "./module.json" assert { type: "json"}; %s.executed = true;' % test
    else:
      content = b'import "./not_found.json" assert { type: "json"}; %s.test.step(function() { assert_unreached("404 script should not be executed"); });' % test

    return status, headers, content
