// META: title=Tests that redirects between origins are allowed when access control is involved.
// META: script=/common/get-host-info.sub.js

    function runSync(test, url)
    {
      const xhr = new XMLHttpRequest();
      xhr.open("GET", url, false);
      xhr.send();
      assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.");
      test.done();
    }
    function runAsync(test, url)
    {
      const xhr = new XMLHttpRequest();
      xhr.open("GET", url, true);
      xhr.onload = test.step_func_done(function() {
        assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.");
      });
      xhr.onerror = test.unreached_func("Network error");
      xhr.send();
      test.done();
    }
    test(t => {
      runSync(t, "resources/redirect-cors.py?location=" + get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow.py")
    }, "Local sync redirect to remote origin");
    async_test(t => {
      runAsync(t, "resources/redirect-cors.py?location=" + get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow.py")
    }, "Local async redirect to remote origin");
    test(t => {
      runSync(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/redirect-cors.py?location=" + get_host_info().HTTP_ORIGIN +
          "/xhr/resources/access-control-basic-allow.py&allow_origin=true")
    }, "Remote sync redirect to local origin");
    async_test(t => {
      runAsync(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/redirect-cors.py?location=" + get_host_info().HTTP_ORIGIN +
          "/xhr/resources/access-control-basic-allow.py&allow_origin=true")
    }, "Remote async redirect to local origin");
    test(t => {
      runSync(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/redirect-cors.py?location=" + get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow.py&allow_origin=true")
    }, "Remote sync redirect to same remote origin");
    async_test(t => {
      runAsync(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/redirect-cors.py?location=" + get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow.py&allow_origin=true")
    }, "Remote async redirect to same remote origin");
