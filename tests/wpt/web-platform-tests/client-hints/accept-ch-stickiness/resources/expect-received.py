def main(request, response):
    """
    Check that headers sent to navigate here contain the device-memory client
    hint, and report success/failure in a way compatible with
    verify_navigation_state() in accept-ch-test.js
    """

    if "device-memory" in request.headers:
      result = "PASS"
    else:
      result = "FAIL"

    content = '''
<script>
  window.opener.postMessage("%s" , "*");
</script>
''' % (result)
    headers = [("Content-Type", "text/html")]
    return 200, headers, content
