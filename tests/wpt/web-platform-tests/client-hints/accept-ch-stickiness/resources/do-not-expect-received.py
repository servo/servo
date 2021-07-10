def main(request, response):
    """
    Check that headers sent to navigate here *do not* contain the device-memory client
    hint, and report success/failure in a way compatible with
    verify_subresource_state() in accept-ch-test.js
    """

    if b"device-memory" in request.headers:
      result = u"FAIL"
    else:
      result = u"PASS"

    content = u'''
<script>
  window.opener.postMessage("%s" , "*");
</script>
''' % (result)
    headers = [(b"Content-Type", b"text/html"), (b"Access-Control-Allow-Origin", b"*")]
    return 200, headers, content
