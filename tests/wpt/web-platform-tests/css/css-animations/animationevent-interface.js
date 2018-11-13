(function() {
  test(function() {
    var event = new AnimationEvent("");
    assert_true(event instanceof window.AnimationEvent);
  }, "the event is an instance of AnimationEvent");

  test(function() {
    var event = new AnimationEvent("");
    assert_true(event instanceof window.Event);
  }, "the event inherts from Event");

  test(function() {
    assert_throws(new TypeError(), function() {
      new AnimationEvent();
    }, 'First argument is required, so was expecting a TypeError.');
  }, 'Missing type argument');

  test(function() {
    var event = new AnimationEvent("test");
    assert_equals(event.type, "test");
  }, "type argument is string");

  test(function() {
    var event = new AnimationEvent(null);
    assert_equals(event.type, "null");
  }, "type argument is null");

  test(function() {
    var event = new AnimationEvent(undefined);
    assert_equals(event.type, "undefined");
  }, "event type set to undefined");

  test(function() {
    var event = new AnimationEvent("test");
    assert_equals(event.animationName, "");
  }, "animationName has default value of empty string");

  test(function() {
    var event = new AnimationEvent("test");
    assert_equals(event.elapsedTime, 0.0);
  }, "elapsedTime has default value of 0.0");

  test(function() {
    var event = new AnimationEvent("test");
    assert_readonly(event, "animationName", "readonly attribute value");
  }, "animationName is readonly");

  test(function() {
    var event = new AnimationEvent("test");
    assert_readonly(event, "elapsedTime", "readonly attribute value");
  }, "elapsedTime is readonly");

  test(function() {
    var event = new AnimationEvent("test", null);
    assert_equals(event.animationName, "");
    assert_equals(event.elapsedTime, 0.0);
  }, "animationEventInit argument is null");

  test(function() {
    var event = new AnimationEvent("test", undefined);
    assert_equals(event.animationName, "");
    assert_equals(event.elapsedTime, 0.0);
  }, "animationEventInit argument is undefined");

  test(function() {
    var event = new AnimationEvent("test", {});
    assert_equals(event.animationName, "");
    assert_equals(event.elapsedTime, 0.0);
  }, "animationEventInit argument is empty dictionary");

  test(function() {
    var event = new AnimationEvent("test", {pseudoElement: "::testPseudo"});
    assert_equals(event.pseudoElement, "::testPseudo");
  }, "AnimationEvent.pseudoElement initialized from the dictionary");

  test(function() {
    var event = new AnimationEvent("test", {animationName: "sample"});
    assert_equals(event.animationName, "sample");
  }, "animationName set to 'sample'");

  test(function() {
    var event = new AnimationEvent("test", {elapsedTime: 0.5});
    assert_equals(event.elapsedTime, 0.5);
  }, "elapsedTime set to 0.5");

  test(function() {
    var eventInit = {animationName: "sample", elapsedTime: 0.5};
    var event = new AnimationEvent("test", eventInit);
    assert_equals(event.animationName, "sample");
    assert_equals(event.elapsedTime, 0.5);
  }, "AnimationEventInit properties set value");
})();
