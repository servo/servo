// META: title=Tests CORS with Access-Control-Allow-Origin header
// META: script=/common/get-host-info.sub.js

    test(function() {
      const xhr = new XMLHttpRequest;

      xhr.open("GET", get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/access-control-basic-allow.py", false);

      xhr.send();

      assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.");
    }, "Allow basic");
