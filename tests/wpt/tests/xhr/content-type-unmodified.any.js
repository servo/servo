"use strict";

async_test(t => {
  const xhr = new XMLHttpRequest();

  xhr.open("POST", "resources/echo-content-type.py");
  xhr.setRequestHeader("content-type", "application/json; charset=UTF-8");

  xhr.onerror = t.unreached_func("Error occurred.");

  xhr.onload = t.step_func_done(() => {
    assert_equals(xhr.status, 200);
    assert_equals(xhr.responseText, "application/json; charset=UTF-8");
  });
  xhr.send("{ \"x\":\"a\"}");
});
