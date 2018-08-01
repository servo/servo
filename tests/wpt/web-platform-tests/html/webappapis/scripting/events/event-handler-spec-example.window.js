var objects = [{}, function() {}, new Number(42), new String()];
var primitives = [42, null, undefined, ""];
objects.forEach(function(object) {
  test(function() {
    var i = 0;
    var uncalled = "assert_unreached('First event handler.');"
    var button = document.createElement('button');
    button.onclick = object; // event handler listener is registered here
    assert_equals(button.onclick, object);
    button.addEventListener('click', function () { assert_equals(++i, 2) }, false);
    button.setAttribute('onclick', uncalled);
    button.addEventListener('click', function () { assert_equals(++i, 3) }, false);
    button.onclick = function () { assert_equals(++i, 1); };
    button.addEventListener('click', function () { assert_equals(++i, 4) }, false);
    button.click()
    assert_equals(button.getAttribute("onclick"), uncalled)
    assert_equals(i, 4);
  }, "Event handler listeners should be registered when they are first set to an object value " +
     "(" + format_value(object) + ").");
});
primitives.forEach(function(primitive) {
  test(function() {
    var i = 0;
    var uncalled = "assert_unreached('First event handler.');"
    var button = document.createElement('button');
    button.onclick = primitive;
    assert_equals(button.onclick, null);
    button.addEventListener('click', function () { assert_equals(++i, 1) }, false);
    button.setAttribute('onclick', uncalled); // event handler listener is registered here
    button.addEventListener('click', function () { assert_equals(++i, 3) }, false);
    button.onclick = function () { assert_equals(++i, 2); };
    button.addEventListener('click', function () { assert_equals(++i, 4) }, false);
    button.click()
    assert_equals(button.getAttribute("onclick"), uncalled)
    assert_equals(i, 4);
  }, "Event handler listeners should be registered when they are first set to an object value " +
     "(" + format_value(primitive) + ").");
});
test(function() {
  var i = 0;
  var uncalled = "assert_unreached('First event handler.');"
  var button = document.createElement('button');
  button.addEventListener('click', function () { assert_equals(++i, 1) }, false);
  button.setAttribute('onclick', uncalled); // event handler listener is registered here
  button.addEventListener('click', function () { assert_equals(++i, 3) }, false);
  button.onclick = function () { assert_equals(++i, 2); };
  button.addEventListener('click', function () { assert_equals(++i, 4) }, false);
  button.click()
  assert_equals(button.getAttribute("onclick"), uncalled)
  assert_equals(i, 4);
}, "Event handler listeners should be registered when they are first set to an object value.");
