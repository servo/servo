// META: title=Tests that asynchronous XMLHttpRequests handle redirects according to the CORS standard.
// META: script=/common/get-host-info.sub.js

    function runTest(test, path, credentials, expectSuccess) {
      const xhr = new XMLHttpRequest();
      xhr.withCredentials = credentials;
      xhr.open("GET", "resources/redirect.py?location=" + get_host_info().HTTP_REMOTE_ORIGIN + path, true);

      xhr.onload = test.step_func_done(function() {
        assert_true(expectSuccess);
        assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.");
      });
      xhr.onerror = test.step_func_done(function() {
        assert_false(expectSuccess);
        assert_equals(xhr.status, 0);
      });
      xhr.send(null);
    }

    const withoutCredentials = false;
    const withCredentials = true;
    const succeeds = true;
    const fails = false;

    // Test simple same origin requests that receive cross origin redirects.

    // The redirect response passes the access check.
    async_test(t => {
      runTest(t, "/xhr/resources/access-control-basic-allow-star.py",
          withoutCredentials, succeeds)
    }, "Request without credentials is redirected to a cross-origin response with Access-Control-Allow-Origin=* (with star)");

    // The redirect response fails the access check because credentials were sent.
    async_test(t => {
      runTest(t, "/xhr/resources/access-control-basic-allow-star.py",
          withCredentials, fails)
    }, "Request with credentials is redirected to a cross-origin response with Access-Control-Allow-Origin=* (with star)");

    // The redirect response passes the access check.
    async_test(t => {
      runTest(t, "/xhr/resources/access-control-basic-allow.py",
          withoutCredentials, succeeds)
    }, "Request without credentials is redirected to a cross-origin response with a specific Access-Control-Allow-Origin");

    // The redirect response passes the access check.
    async_test(t => {
      runTest(t, "/xhr/resources/access-control-basic-allow.py",
          withCredentials, succeeds)
    }, "Request with credentials is redirected to a cross-origin response with a specific Access-Control-Allow-Origin");

    // forbidding credentials. The redirect response passes the access check.
    async_test(t => {
      runTest(t, "/xhr/resources/access-control-basic-allow-no-credentials.py",
          withoutCredentials, succeeds)
    }, "Request without credentials is redirected to a cross-origin response with a specific Access-Control-Allow-Origin (no credentials)");

    // forbidding credentials. The redirect response fails the access check.
    async_test(t => {
      runTest(t, "/xhr/resources/access-control-basic-allow-no-credentials.py",
          withCredentials, fails)
    }, "Request with credentials is redirected to a cross-origin response with a specific Access-Control-Allow-Origin (no credentials)");
