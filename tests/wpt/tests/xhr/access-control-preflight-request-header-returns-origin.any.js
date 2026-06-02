// META: title=Access-Control-Request-Origin accept different origin between preflight and actual request
// META: script=/common/get-host-info.sub.js
"use strict";

async_test(t => {
  const xhr = new XMLHttpRequest();

  xhr.open("GET", corsURL("resources/access-control-preflight-request-header-returns-origin.py"));

  xhr.setRequestHeader("X-Test", "foobar");

  xhr.onerror = t.unreached_func("Error occurred.");

  xhr.onload = t.step_func_done(() => {
    assert_equals(xhr.status, 200);
    assert_equals(xhr.responseText, "PASS");
  });

  xhr.send();
});

function corsURL(path) {
  const url = new URL(path, location.href);
  url.hostname = get_host_info().REMOTE_HOST;
  return url.href;
}
