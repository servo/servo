def main(request, response):
    """Send a response with the Origin-Policy header asking for the latest
      policy, that runs the test JS given by the ?test= argument. This is meant
       to be loaded into an iframe by origin-policy-test-runner.js.

       The ?test= argument is best given as an absolute path (starting with /)
       since it will otherwise be interpreted relative to where this file is
       served.
    """
    test_file = request.GET.first(b"test")

    expected_ids = request.GET.first(b"expectedIds")

    response.headers.set(b"Origin-Policy", b"allowed=(latest)")
    response.headers.set(b"Content-Type", b"text/html")

    ret_val = b"""
    <!DOCTYPE html>
    <meta charset="utf-8">
    <title>Origin policy subframe</title>

    <script src="/resources/testharness.js"></script>

    <div id="log"></div>

    <script type="module" src="%s"></script>
  """ % test_file

    if expected_ids != b"undefined":
      ret_val += b"""
      <script type="module">
        test(() => {
          assert_array_equals(originPolicyIds, %s);
        }, "Expected originPolicyIDs check");
      </script>
      """ % expected_ids

    return ret_val
