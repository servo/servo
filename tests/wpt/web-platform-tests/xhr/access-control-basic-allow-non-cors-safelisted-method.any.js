// META: title=Tests cross-origin request with non-CORS-safelisted method
// META: script=/common/get-host-info.sub.js

    test(function() {
      const xhr = new XMLHttpRequest;

      xhr.open("PUT", get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/access-control-basic-put-allow.py", false);

      xhr.setRequestHeader("Content-Type", "text/plain; charset=UTF-8");

      xhr.send("PASS: PUT data received");

      assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.\nPASS: PUT data received");
    }, "Allow PUT request");
