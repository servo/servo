// META: title=Preflight cache should allow second request without preflight OPTIONS request
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js

    const uuid = token();

    async_test(function(test) {
      const xhr = new XMLHttpRequest;
      xhr.onerror = test.unreached_func("FAIL: Network error.");
      xhr.onload = test.step_func(function() {
        // Token reset.  We can start the test now.
        assert_equals(xhr.responseText, "PASS");
        firstRequest();
      });

      xhr.open("GET", get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/reset-token.py?token=" + uuid, true);
      xhr.send();

      function firstRequest() {
        xhr.onload = test.step_func(function() {
          assert_equals(xhr.responseText, "PASS: First PUT request.");
          secondRequest();
        });
        xhr.open("PUT", get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/access-control-basic-preflight-cache.py?token=" + uuid, true);
        xhr.send();
      }

      function secondRequest() {
        xhr.onload = test.step_func_done(function() {
          assert_equals(xhr.responseText, "PASS: Second PUT request. Preflight worked.");
        });
        xhr.open("PUT", get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/access-control-basic-preflight-cache.py?token=" + uuid, true);
        xhr.send();
      }
    }, "Preflight cache should allow second request");
