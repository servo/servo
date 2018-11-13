async_test(t => {
  const client = new XMLHttpRequest();
  client.onload = t.step_func_done(() => {
    assert_equals(client.getResponseHeader("foo-test"), "1, 2, 3");
  });
  client.onerror = t.unreached_func("unexpected error");
  client.open("GET", "resources/headers-basic.asis");
  client.send();
}, "getResponseHeader('foo-test')");

async_test(t => {
  const client = new XMLHttpRequest();
  client.onload = t.step_func_done(() => {
    assert_equals(client.getResponseHeader("www-authenticate"), "1, 2, 3, 4");
  });
  client.onerror = t.unreached_func("unexpected error");
  client.open("GET", "resources/headers-www-authenticate.asis");
  client.send();
}, "getResponseHeader('www-authenticate')");
