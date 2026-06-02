// META: title=XMLHttpRequest: loadstart event

var test = async_test();
test.step(function () {
  var client = new XMLHttpRequest();
  client.onloadstart = test.step_func(function (e) {
    assert_true(e instanceof ProgressEvent);
    assert_equals(e.type, "loadstart");
    assert_equals(client.readyState, 1);
    test.done();
  });
  test.step_timeout(function () {
    assert_unreached("onloadstart not called after 500 ms");
  }, 500);
  client.open("GET", "resources/well-formed.xml");
  client.send(null);
});
