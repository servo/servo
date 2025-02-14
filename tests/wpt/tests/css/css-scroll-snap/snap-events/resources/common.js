function checkSnapEventSupport(event_type) {
  if (event_type == "scrollsnapchange") {
    assert_true(window.onscrollsnapchange !== undefined, "scrollsnapchange not supported");
  } else if (event_type == "scrollsnapchanging") {
    assert_true(window.onscrollsnapchanging !== undefined, "scrollsnapchanging not supported");
  } else {
    assert_unreached(`Unknown snap event type selected: ${event_type}`);
  }
}

function assertSnapEvent(evt, expected_ids) {
  assert_equals(evt.bubbles, evt.target == document,
    "snap events don't bubble except when fired at the document");
  assert_false(evt.cancelable, "snap events are not cancelable.");
  assert_equals(evt.snapTargetBlock, expected_ids.block,
    "snap event supplied expected target in block axis");
  assert_equals(evt.snapTargetInline, expected_ids.inline,
    "snap event supplied expected target in inline axis");
}

async function snap_test_setup(test, scroller, event_type) {
  checkSnapEventSupport(event_type);
  await waitForScrollReset(test, scroller);
  await waitForCompositorCommit();
  test.add_cleanup(async () => {
    await waitForScrollReset(test, scroller);
  });
}

async function test_snap_event(test, test_data, event_type,
                               use_onsnap_member = false) {
  await snap_test_setup(test, test_data.scroller, event_type);

  let listener = test_data.scroller ==
    document.scrollingElement ? document : test_data.scroller;

  const event_promise = waitForSnapEvent(listener, event_type, true,
                                         use_onsnap_member);
  await test_data.scrolling_function();
  let evt = await event_promise;

  assertSnapEvent(evt, test_data.expected_snap_targets);
  assert_approx_equals(test_data.scroller.scrollTop,
    test_data.expected_scroll_offsets.y, 1,
    "vertical scroll offset mismatch.");
  assert_approx_equals(test_data.scroller.scrollLeft,
    test_data.expected_scroll_offsets.x, 1,
    "horizontal scroll offset mismatch.");
}

async function test_scrollsnapchange(test, test_data, use_onsnap_member = false) {
  await test_snap_event(test, test_data, "scrollsnapchange", use_onsnap_member);
}

function waitForEventUntil(event_target, event_type, wait_until,
                           use_onsnap_member = false) {
  return new Promise(resolve => {
    let result = null;
    const listener = (evt) => {
      result = evt;
    };
    if (use_onsnap_member) {
      if (event_type === "scrollsnapchanging") {
        event_target.onscrollsnapchanging = listener;
      } else {
        event_target.onscrollsnapchange = listener;
      }
    } else {
      event_target.addEventListener(event_type, listener);
    }
    wait_until.then(() => {
      if (use_onsnap_member) {
        if (event_type === "scrollsnapchanging") {
          event_target.onscrollsnapchanging = null;
        } else {
          event_target.onscrollsnapchange = null;
        }
      } else {
        event_target.removeEventListener(event_type, listener);
      }
      resolve(result);
    });
  });
}

function waitForEventsUntil(event_target, event_type, wait_until) {
  return new Promise(resolve => {
    let result = [];
    const listener = (evt) => {
      result.push(evt);
    };
    event_target.addEventListener(event_type, listener);
    wait_until.then(() => {
      event_target.removeEventListener(event_type, listener);
      resolve(result);
    });
  });
}

// Proxy a wait for a snap event. We want to avoid having a test
// timeout in the event of an expected snap event not firing in a particular
// test case as that would cause the entire file to fail.
// Snap events should fire before scrollend, so if a scroll should happen, wait
// for a scrollend event. Otherwise, just do a rAF-based wait.
function waitForSnapEvent(event_target, event_type, scroll_happens = true,
                          use_onsnap_member = false) {
  return scroll_happens ? waitForEventUntil(event_target, event_type,
                                   waitForScrollendEventNoTimeout(event_target),
                                   use_onsnap_member)
                        : waitForEventUntil(event_target, event_type,
                                   waitForAnimationFrames(2),
                                   use_onsnap_member);
}

function waitForScrollSnapChangeEvent(event_target, scroll_happens = true) {
  return waitForSnapEvent(event_target, "scrollsnapchange", scroll_happens);
}

function getScrollbarToScrollerRatio(scroller) {
  // Ideally we'd subtract the length of the scrollbar thumb from
  // the dividend but there isn't currently a way to get the
  // scrollbar thumb length.
  return scroller.clientHeight /
      (scroller.scrollHeight - scroller.clientHeight);
}
