// META: title=XMLHttpRequest: loadend event

var test = async_test();
test.step(function () {
  var client = new XMLHttpRequest();
  client.onloadend = test.step_func(function (e) {
    assert_true(e instanceof ProgressEvent);
    assert_equals(e.type, "loadend");
    test.done();
  });
  client.onreadystatechange = function () {
    if (client.readyState !== 4) return;
    test.step_timeout(() => {
      assert_unreached("onloadend not called after 100 ms");
    }, 100);
  };
  client.open("GET", "resources/well-formed.xml");
  client.send(null);
});
