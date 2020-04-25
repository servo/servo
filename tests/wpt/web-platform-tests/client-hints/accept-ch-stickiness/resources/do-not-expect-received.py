def main(request, response):
    """
    Check that headers sent to navigate here *do not* contain the device-memory client
    hint, and report success/failure in a way compatible with
    verify_subresource_state() in accept-ch-test.js
    """

    if "device-memory" in request.headers:
      result = "FAIL"
    else:
      result = "PASS"

    content = '''
<script>
  window.opener.postMessage("%s" , "*");
</script>
''' % (result)
    headers = [("Content-Type", "text/html")]
    return 200, headers, content
