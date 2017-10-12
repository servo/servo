import re

def main(request, response):
    headers = [("Content-Type", "text/javascript")]
    test = request.GET.first('test')
    assert(re.match('^[a-zA-Z0-9_]+$', test));

    if test.find('_load') >= 0:
      status = 200
      content = '"use strict"; %s.executed = true;' % test
    else:
      status = 404
      content = '"use strict"; %s.test.step(function() { assert_unreached("404 script should not be executed"); });' % test

    return status, headers, content
