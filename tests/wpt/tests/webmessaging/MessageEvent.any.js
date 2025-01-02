// META: title=MessageEvent

var prefixes = ['moz', 'ms', 'o', 'webkit'];
prefixes.forEach(function(prefix) {
  var name = prefix + "InitMessageEvent";

  test(function() {
    assert_false(name in MessageEvent.prototype);
  }, name + " on the prototype");

  test(function() {
    var event = new MessageEvent("message");
    assert_false(name in event);
  }, name + " on the instance");
});

test(function() {
  var event = new MessageEvent("message");
  assert_throws_js(TypeError, function() {
    event.initMessageEvent();
  }, "Not enough arguments to initMessageEvent");
}, "initMessageEvent with no arguments");
