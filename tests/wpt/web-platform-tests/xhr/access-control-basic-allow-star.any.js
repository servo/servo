// META: title=Tests "*" setting for Access-Control-Allow-Origin header
// META: script=/common/get-host-info.sub.js

    const xhr = new XMLHttpRequest;

    test(function(test) {
      xhr.open("GET", get_host_info().HTTP_REMOTE_ORIGIN + "/xhr/resources/access-control-basic-allow-star.py", false);

      xhr.send();

      assert_equals(xhr.responseText, "PASS: Cross-domain access allowed.");
    }, "Allow star");
