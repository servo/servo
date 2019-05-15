// META: title=Tests cross-origin async request with non-CORS-safelisted method
// META: script=/common/get-host-info.sub.js

    async_test((test) => {
      const xhr = new XMLHttpRequest;

      xhr.onload = test.step_func_done(() => {
        assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.\nPASS: PUT data received");
      });

      xhr.onerror = test.unreached_func("Unexpected error.");

      xhr.open("PUT", get_host_info().HTTP_REMOTE_ORIGIN +
          "/xhr/resources/access-control-basic-put-allow.py");
      xhr.setRequestHeader("Content-Type", "text/plain; charset=UTF-8");
      xhr.send("PASS: PUT data received");
    }, "Allow async PUT request");
