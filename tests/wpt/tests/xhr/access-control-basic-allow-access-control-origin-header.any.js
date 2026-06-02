// META: title=Access control test with origin header
// META: script=/common/get-host-info.sub.js

    async_test(function(test) {
      const xhr = new XMLHttpRequest;

      xhr.open("GET", get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/access-control-origin-header.py", false);
      xhr.send();

      assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.\n" +
          "HTTP_ORIGIN: " + get_host_info().HTTP_ORIGIN);
      test.done();
    }, "Access control test with origin header");
