promise_test(async t => {
  const response = await fetch("resources/identical-duplicates.asis");
  assert_equals(response.statusText, "BLAH");
  assert_equals(response.headers.get("test"), "x, x");
  assert_equals(response.headers.get("content-type"), "text/plain, text/plain");
  assert_equals(response.headers.get("content-length"), "6, 6");
  const text = await response.text();
  assert_equals(text, "Test.\n");
}, "fetch() and duplicate Content-Length/Content-Type headers");

async_test(t => {
  const xhr = new XMLHttpRequest();
  xhr.open("GET", "resources/identical-duplicates.asis");
  xhr.send();
  xhr.onload = t.step_func_done(() => {
    assert_equals(xhr.statusText, "BLAH");
    assert_equals(xhr.getResponseHeader("test"), "x, x");
    assert_equals(xhr.getResponseHeader("content-type"), "text/plain, text/plain");
    assert_equals(xhr.getResponseHeader("content-length"), "6, 6");
    assert_equals(xhr.getAllResponseHeaders(), "content-length: 6, 6\r\ncontent-type: text/plain, text/plain\r\ntest: x, x\r\n");
    assert_equals(xhr.responseText, "Test.\n");
  });
}, "XMLHttpRequest and duplicate Content-Length/Content-Type headers");
