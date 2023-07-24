// META: title=Tests that asynchronous XMLHttpRequests handle redirects according to the CORS standard.
// META: script=/common/get-host-info.sub.js

    function runTest(test, destination, parameters, customHeader, local, expectSuccess) {
      const xhr = new XMLHttpRequest();
      const url = (local ? get_host_info().HTTP_ORIGIN : get_host_info().HTTP_REMOTE_ORIGIN) +
        "/xhr/resources/redirect-cors.py?location=" + destination + "&" +  parameters;

      xhr.open("GET", url, true);

      if (customHeader)
        xhr.setRequestHeader("x-test", "test");

      xhr.onload = test.step_func_done(function() {
        assert_true(expectSuccess);
        assert_true(xhr.responseText.startsWith("PASS"));
      });
      xhr.onerror = test.step_func_done(function() {
        assert_false(expectSuccess);
        assert_equals(xhr.status, 0);
      });
      xhr.send();
    }

    const withCustomHeader = true;
    const withoutCustomHeader = false;
    const local = true;
    const remote = false;
    const succeeds = true;
    const fails = false;

    // Test simple cross origin requests that receive redirects.

    // The redirect response fails the access check because the redirect lacks a CORS header.
    async_test(t => {
      runTest(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow-star.py", "",
          withoutCustomHeader, remote, fails)
    }, "Request is redirected without CORS headers to a response with Access-Control-Allow-Origin=*");

    // The redirect response passes the access check.
    async_test(t => {
      runTest(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow-star.py", "allow_origin=true",
          withoutCustomHeader, remote, succeeds)
    }, "Request is redirected to a response with Access-Control-Allow-Origin=*");

    // The redirect response fails the access check because user info was sent.
    async_test(t => {
      runTest(t, get_host_info().HTTP_REMOTE_ORIGIN.replace("http://", "http://username:password@") +
          "/xhr/resources/access-control-basic-allow-star.py", "allow_origin=true",
          withoutCustomHeader, remote, fails)
    }, "Request with user info is redirected to a response with Access-Control-Allow-Origin=*");

    // The redirect response fails the access check because the URL scheme is unsupported.
    async_test(t => {
      runTest(t, "foo://bar.cgi", "allow_origin=true", withoutCustomHeader, remote, fails)
    }, "Request is redirect to a bad URL");

    // The preflighted redirect response fails the access check because of preflighting.
    async_test(t => {
      runTest(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow-star.py",
          "allow_origin=true&redirect_preflight=true", withCustomHeader, remote, fails)
    }, "Preflighted request is redirected to a response with Access-Control-Allow-Origin=*");

    // The preflighted redirect response fails the access check after successful preflighting.
    async_test(t => {
      runTest(t, get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-allow-star.py",
          "allow_origin=true&allow_header=x-test&redirect_preflight=true",
          withCustomHeader, remote, fails)
    }, "Preflighted request is redirected to a response with Access-Control-Allow-Origin=* and header allowed");

    // The same-origin redirect response passes the access check.
    async_test(t => {
      runTest(t, get_host_info().HTTP_ORIGIN + "/xhr/resources/pass.txt",
          "", withCustomHeader, local, succeeds)
    }, "Request is redirected to a same-origin resource file");
