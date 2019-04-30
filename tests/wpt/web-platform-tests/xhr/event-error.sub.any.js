// META: title=XMLHttpRequest Test: event - error

async_test(function (t) {
  var client = new XMLHttpRequest();
  client.onerror = t.step_func(function (e) {
    assert_true(e instanceof ProgressEvent);
    assert_equals(e.type, "error");
    t.done();
  });

  client.open("GET", "http://nonexistent.{{host}}:{{ports[http][0]}}");
  client.send("null");
});