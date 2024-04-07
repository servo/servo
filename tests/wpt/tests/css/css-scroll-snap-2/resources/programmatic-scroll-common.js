// Helper functions for snapchanged-on-programmatic-* tests.

// Utility function to test that onsnapchanging is triggered for
// snapchanging-on-programmatic-* tests which set up a similar layout in which
// the |scroller| has 3 snap targets that form a vertical column along
// |scroller|'s middle. onsnapchanging should be triggered by conducting a
// programmatic scroll to the top of snap_target.
async function test_programmatic_scroll_onsnapchanging(test,
                                                       scroller,
                                                       event_target,
                                                       snap_target) {
  await snap_test_setup(test, scroller, "snapchanging");
  const expected_snap_targets = { block: snap_target, inline: null };

  // Scroll and wait for a snapchanging event.
  const snapchanging_promise = waitForOnSnapchanging(event_target);
  scroller.scrollTo(0, snap_target.offsetTop);
  const snapchanging_event = await snapchanging_promise;

  // Assert that snapchanging fired and indicated that snap_target would
  // be snapped to.
  assertSnapEvent(snapchanging_event, expected_snap_targets);
  assert_equals(scroller.scrollLeft, 0, "scrollLeft is zero");
  assert_equals(scroller.scrollTop, snap_target.offsetTop,
    "snapped to snap_target");
}
