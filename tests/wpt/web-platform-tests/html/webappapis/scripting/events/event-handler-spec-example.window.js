var objects = [{}, function() {}, new Number(42), new String()];
var primitives = [42, null, undefined, ""];
var firstEventHandler;
objects.forEach(function(object) {
  test(t => {
    var i = 0;
    firstEventHandler = t.unreached_func('First event handler.');
    var uncalled = "firstEventHandler();";
    var button = document.createElement('button');
    button.onclick = object; // event handler listener is registered here
    assert_equals(button.onclick, object);
    button.addEventListener('click', t.step_func(() => { assert_equals(++i, 2) }), false);
    button.setAttribute('onclick', uncalled);
    button.addEventListener('click', t.step_func(() => { assert_equals(++i, 3) }), false);
    button.onclick = t.step_func(() => { assert_equals(++i, 1); });
    button.addEventListener('click', t.step_func(() => { assert_equals(++i, 4) }), false);
    button.click()
    assert_equals(button.getAttribute("onclick"), uncalled)
    assert_equals(i, 4);
  }, "Event handler listeners should be registered when they are first set to an object value " +
     "(" + format_value(object) + ").");
});
primitives.forEach(function(primitive) {
  test(t => {
    var i = 0;
    firstEventHandler = t.unreached_func('First event handler.');
    var uncalled = "firstEventHandler();";
    var button = document.createElement('button');
    button.onclick = primitive;
    assert_equals(button.onclick, null);
    button.addEventListener('click', t.step_func(() => { assert_equals(++i, 1) }), false);
    button.setAttribute('onclick', uncalled); // event handler listener is registered here
    button.addEventListener('click', t.step_func(() => { assert_equals(++i, 3) }), false);
    button.onclick = t.step_func(() => { assert_equals(++i, 2); });
    button.addEventListener('click', t.step_func(() => { assert_equals(++i, 4) }), false);
    button.click()
    assert_equals(button.getAttribute("onclick"), uncalled)
    assert_equals(i, 4);
  }, "Event handler listeners should be registered when they are first set to an object value " +
     "(" + format_value(primitive) + ").");
});
test(t => {
  var i = 0;
  firstEventHandler = t.unreached_func('First event handler.');
  var uncalled = "firstEventHandler();";
  var button = document.createElement('button');
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 1) }), false);
  button.setAttribute('onclick', uncalled); // event handler listener is registered here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 3) }), false);
  button.onclick = t.step_func(() => { assert_equals(++i, 2); });
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 4) }), false);
  button.click()
  assert_equals(button.getAttribute("onclick"), uncalled)
  assert_equals(i, 4);
}, "Event handler listeners should be registered when they are first set to an object value.");
