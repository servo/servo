def main(request, response):
    """
    Check that headers sent to navigate here contain the device-memory client
    hint, and report success/failure in a way compatible with
    verify_navigation_state() in accept-ch-test.js
    """

    if b"sec-ch-device-memory" not in request.headers:
      result = u"DEVICE-MEMORY"
    elif b"device-memory" not in request.headers:
      result = u"DEVICE-MEMORY-DEPRECATED"
    elif b"sec-ch-ua" not in request.headers:
      result = u"UA"
    elif b"sec-ch-ua-mobile" not in request.headers:
      result = u"MOBILE"
    elif b"sec-ch-ua-platform" not in request.headers:
      result = u"PLATFORM"
    else:
      result = u"PASS"

    content = u'''
<script>
  let messagee = window.opener || window.parent;
  messagee.postMessage("%s" , "*");
</script>
''' % (result)
    headers = [(b"Content-Type", b"text/html"), (b"Access-Control-Allow-Origin", b"*")]
    return 200, headers, content
