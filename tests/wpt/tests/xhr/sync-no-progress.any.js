// META: timeout=long
test(t => {
  let xhr = new XMLHttpRequest();
  let loadEventFired = false;
  xhr.onprogress = t.unreached_func('progress event should not be fired');
  xhr.onload = () => {
    loadEventFired = true;
  };
  xhr.open('GET', 'resources/trickle.py?count=4&delay=150', false);
  xhr.send();
  // Check the load event as a sanity check that the test is working.
  assert_true(loadEventFired, 'load event should have fired');
}, 'progress event should not be fired by sync XHR');
