def main(request, response):
    """
    Check that headers sent to navigate here contain the device-memory client
    hint, and report success/failure in a way compatible with
    verify_navigation_state() in accept-ch-test.js
    """

    if "device-memory" in request.headers and "sec-ch-ua" in request.headers and "sec-ch-ua-mobile" in request.headers:
      result = "PASS"
    else:
      result = "FAIL"

    content = '''
<script>
  let messagee = window.opener || window.parent;
  messagee.postMessage("%s" , "*");
</script>
''' % (result)
    headers = [("Content-Type", "text/html"), ("Access-Control-Allow-Origin", "*")]
    return 200, headers, content
