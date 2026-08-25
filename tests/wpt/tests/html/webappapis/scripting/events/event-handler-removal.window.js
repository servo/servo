let firstEventHandler;

test(t => {
  var i = 0;
  firstEventHandler = t.unreached_func('First event handler.');
  var uncalled = "firstEventHandler();";
  var button = document.createElement('button');
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 1) }), false);
  button.setAttribute('onclick', uncalled);                             // event handler is activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 2) }), false);
  button.onclick = null;                                                // but de-activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 3) }), false);
  button.onclick = t.step_func(() => { assert_equals(++i, 4); });       // and re-activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 5) }), false);
  button.click()
  assert_equals(button.getAttribute("onclick"), uncalled)
  assert_equals(i, 5);
}, "Event handler set through content attribute should be removed when they are set to null.");

let happened = 0;
test(() => {
  var script = "happened++;";
  var button = document.createElement('button');
  button.setAttribute('onclick', script);                               // event handler is activated here
  button.onclick = null;                                                // but de-activated here
  assert_equals(button.getAttribute("onclick"), script)
  button.setAttribute('onclick', script);                               // and re-activated here
  button.click()
  assert_equals(happened, 1);
}, "Event handler set through content attribute should be re-activated even if content is the same.");

test(t => {
  var i = 0;
  firstEventHandler = t.unreached_func('First event handler.');
  var uncalled = "firstEventHandler();";
  var button = document.createElement('button');
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 1) }), false);
  button.setAttribute('onclick', uncalled);                             // event handler is activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 2) }), false);
  button.removeAttribute('onclick');                                    // but de-activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 3) }), false);
  button.onclick = t.step_func(() => { assert_equals(++i, 4); });       // and re-activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 5) }), false);
  button.click()
  assert_equals(i, 5);
}, "Event handler set through content attribute should be deactivated when the content attribute is removed.");
test(t => {
  var i = 0;
  firstEventHandler = t.unreached_func('First event handler.');
  var uncalled = "firstEventHandler();";
  var button = document.createElement('button');
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 1) }), false);
  button.onclick = t.unreached_func('First event handler.');            // event handler is activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 2) }), false);
  button.onclick = null;                                                // but de-activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 3) }), false);
  button.onclick = t.step_func(() => { assert_equals(++i, 4); });       // and re-activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 5) }), false);
  button.click()
  assert_equals(i, 5);
}, "Event handler set through IDL should be deactivated when the IDL attribute is set to null.");
test(t => {
  var i = 0;
  firstEventHandler = t.unreached_func('First event handler.');
  var uncalled = "firstEventHandler();";
  var button = document.createElement('button');
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 1) }), false);
  button.onclick = t.unreached_func('First event handler.');            // event handler is activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 3) }), false);
  button.removeAttribute('onclick');                                    // and NOT de-activated here
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 4) }), false);
  button.onclick = t.step_func(() => { assert_equals(++i, 2); });
  button.addEventListener('click', t.step_func(() => { assert_equals(++i, 5) }), false);
  button.click()
  assert_equals(i, 5);
}, "Event handler set through IDL should NOT be deactivated when the content attribute is removed.");
