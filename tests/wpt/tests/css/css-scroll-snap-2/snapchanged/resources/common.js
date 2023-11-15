function checkSnapchangedSupport() {
  assert_true(window.onsnapchanged !== undefined, "snapchanged not supported");
}

function assertSnapchangedEvent(evt, expected_ids) {
  assert_equals(evt.bubbles, false, "snapchanged event doesn't bubble");
  assert_false(evt.cancelable, "snapchanged event is not cancelable.");
  const actual = Array.from(evt.snapTargets, el => el.id).join(",");
  const expected = expected_ids.join(",");
  assert_equals(actual, expected, "snapped to expected targets");
}

async function test_snapchanged(test, test_data) {
  checkSnapchangedSupport();
  await waitForScrollReset(test, test_data.scroller);

  let listener = test_data.scroller ==
      document.scrollingElement ? document : test_data.scroller;

  const snapchanged_promise = waitForSnapChangedEvent(listener);
  await test_data.scrolling_function();
  let evt = await snapchanged_promise;

  assertSnapchangedEvent(evt,
      test_data.expected_snap_targets);
  assert_equals(test_data.scroller.scrollTop,
    test_data.expected_scroll_offsets.y,
    "vertical scroll offset mismatch.");
  assert_equals(test_data.scroller.scrollLeft,
    test_data.expected_scroll_offsets.x,
    "horizontal scroll offset mismatch.");
}

function waitForEventUntil(event_target, event_type, wait_until) {
  return new Promise(resolve => {
    let result = null;
    const listener = (evt) => {
      result = evt;
    };
    event_target.addEventListener(event_type, listener);
    wait_until.then(() => {
      event_target.removeEventListener(event_type, listener);
      resolve(result);
    });
  });
}

// Proxy a wait for a snapchanged event. We want to avoid having a test
// timeout in the event of an expected snapchanged not firing in a particular
// test case as that would cause the entire file to fail.
// Snapchanged should fire before scrollend, so if a scroll should happen, wait
// for a scrollend event. Otherwise, just do a rAF-based wait.
function waitForSnapChangedEvent(event_target, scroll_happens = true) {
  return scroll_happens ? waitForEventUntil(event_target, "snapchanged",
                                   waitForScrollendEventNoTimeout(event_target))
                        : waitForEventUntil(event_target, "snapchanged",
                                   waitForAnimationFrames(2));
}
