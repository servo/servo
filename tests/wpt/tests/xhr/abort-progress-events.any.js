// META: title=XMLHttpRequest: abort() and progress timer race condition

// Test for https://bugzilla.mozilla.org/show_bug.cgi?id=1851832
// Ensures that readystatechange and progress events don't fire when the
// state is UNSENT, as required by the spec.

async_test(t => {
  const xhr = new XMLHttpRequest();
  let eventsFiredInUnsent = false;

  // Monitor for events firing in UNSENT state
  xhr.addEventListener('readystatechange', t.step_func(() => {
    if (xhr.readyState === XMLHttpRequest.UNSENT) {
      eventsFiredInUnsent = true;
    }
  }));

  xhr.addEventListener('progress', t.step_func(() => {
    if (xhr.readyState === XMLHttpRequest.UNSENT) {
      eventsFiredInUnsent = true;
    }
  }));

  // Start a request and immediately abort it
  xhr.open('GET', 'resources/pass.txt');
  xhr.send();
  xhr.abort();

  // The spec says when abort() is called on a DONE request, state changes
  // to UNSENT and no readystatechange event is dispatched. Progress events
  // should also not fire in UNSENT state.
  assert_equals(xhr.readyState, XMLHttpRequest.UNSENT,
                'State should be UNSENT immediately after abort');

  // Give time for any progress timer callbacks that might have been queued
  t.step_timeout(() => {
    assert_false(eventsFiredInUnsent,
                 'No events should fire when state is UNSENT');
    t.done();
  }, 200);
}, 'Events must not fire when state is UNSENT after abort()');

async_test(t => {
  const xhr = new XMLHttpRequest();
  let callCount = 0;

  // Calling open/abort in a readystatechange handler can trigger
  // a scenario where the progress timer callback is queued but then
  // the state becomes UNSENT. This should not cause assertions or crashes.
  xhr.addEventListener('readystatechange', t.step_func(e => {
    // Limit recursion to prevent infinite loop
    if (callCount++ < 5) {
      e.currentTarget.open('GET', 'resources/pass.txt');
      e.currentTarget.abort();
    }
  }));

  xhr.open('GET', 'resources/pass.txt');
  xhr.send();

  // Allow time for the test to complete
  t.step_timeout(() => {
    assert_true(callCount > 0, 'readystatechange handler should have been called');
    t.done();
  }, 200);
}, 'Calling open/abort in readystatechange should not crash');
