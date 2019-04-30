// META: title=XMLHttpRequest: The send() method: Fire a progress event named progress (synchronous flag is unset)
// META: timeout=long

var test = async_test();
test.step(function () {
  var client = new XMLHttpRequest();
  client.onprogress = test.step_func(function (e) {
    assert_true(e instanceof ProgressEvent);
    assert_equals(e.type, "progress");
    test.done();
  });
  client.onreadystatechange = test.step_func(function () {
    if (client.readyState === 4)
      assert_unreached("onprogress not called.");
  });
  client.open("GET", "resources/trickle.py?count=4&delay=150");
  client.send(null);
});