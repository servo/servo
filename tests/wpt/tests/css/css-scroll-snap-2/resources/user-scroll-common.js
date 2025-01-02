// Helper functions for scrollsnapchange-on-user-* tests.

// This performs a touch scroll on |scroller| using the coordinates provided
// in |start_pos| and |end_pos|.
// It is meant for use in scrollsnapchange & scrollsnapchanging tests for triggering snap
// events when touch scrolling from |start_pos| to |end_pos|.
function snap_event_touch_scroll_helper(start_pos, end_pos) {
  return new test_driver.Actions()
    .addPointer("TestPointer", "touch")
    .pointerMove(Math.round(start_pos.x), Math.round(start_pos.y))
    .pointerDown()
    .addTick()
    .pause(200)
    .pointerMove(Math.round(end_pos.x), Math.round(end_pos.y))
    .addTick()
    .pointerUp()
    .send();
}

// This drags the provided |scroller|'s scrollbar  vertically by |drag_amt|.
// Snap event tests should provide a |drag_amt| that would result in a
// the desired snap event being triggered.
const vertical_offset_into_scrollbar = 30;
function snap_event_scrollbar_drag_helper(scroller, scrollbar_width, drag_amt) {
  let x, y, bounds;
  if (scroller == document.scrollingElement) {
    bounds = document.documentElement.getBoundingClientRect();
    x = Math.round(window.innerWidth - scrollbar_width / 2);
  } else {
    bounds = scroller.getBoundingClientRect();
    x = Math.round(bounds.right - Math.round(scrollbar_width / 2));
  }
  y = Math.round(bounds.top + vertical_offset_into_scrollbar);
  return new test_driver.Actions()
    .addPointer('TestPointer', 'mouse')
    .pointerMove(x, y)
    .pointerDown()
    .pointerMove(x, Math.round(y + drag_amt))
    .addTick()
    .pointerUp()
    .send();
}

// This tests that snap event of type |event_type| don't fire for a user (wheel)
// scroll that snaps back to the same element. Snap events tests should provide
// a |delta| small enough that no change in |scroller|'s snap targets occurs at
// the end of the scroll.
async function test_no_snap_event(test, scroller, delta, event_type) {
  const listening_element = scroller == document.scrollingElement
      ? document : scroller;
  checkSnapEventSupport(event_type);
  await waitForScrollReset(test, scroller);
  await waitForCompositorCommit();
  let snap_event_promise = waitForSnapEvent(listening_element, event_type);
  // Set the scroll destination to just a little off (0, 0) top so we snap
  // back to the top box.
  await new test_driver.Actions().scroll(0, 0, delta, delta,
      { origin: scroller }).send();
  let evt = await snap_event_promise;
  assert_equals(evt, null, "no snap event since scroller is back to top");
  assert_equals(scroller.scrollTop, 0, "scroller snaps back to the top");
  assert_equals(scroller.scrollLeft, 0, "scroller snaps back to the left");
}

async function test_no_scrollsnapchange(t, scroller, delta) {
  await test_no_snap_event(t, scroller, delta, "scrollsnapchange");
}

async function test_no_scrollsnapchanging(t, scroller, delta) {
  await test_no_snap_event(t, scroller, delta, "scrollsnapchanging");
}
