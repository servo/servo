'use strict';

/**
 * Run assertions to check whether a scroller applies scroll axis lock.
 *
 * @param {test}          The test object that run the assertions.
 * @param {config}        Test configuration
 *   - scroller: Element whose scrolling is tested.
 *   - input_source: the input triggering the scroll ("touch" or "wheel").
 *   - dominant_axis: the dominant axis for the scroll ("X" or "Y").
 *   - delta_dominant_axis_sign: should be +1 if the scroll is performed from
 *     done from top (respectively left) to bottom (respectively right) on the
 *     dominant axis or -1 if it's done the other way around.
 *   - delta_minor_axis_sign: same but for the minor axis.
 * @return {promise}      A promise resolved after the checks complete
 *                        successufully or is rejected in case of failure.
 *
 */
async function check_scroll_axis_lock(test, config) {
  await waitForCompositorReady();

  const scroller = config.scroller;

  let received_wheel_event = null;

  // Scroll mostly along the dominant axis, with a small component along the
  // minor axis.
  // This steep angle would normally trigger railing to the dominant axis.
  // With scroll-axis-lock: none, it should not rail.
  const bounds = scroller.getBoundingClientRect();
  const start_x = bounds.left + bounds.width / 2;
  const start_y = bounds.top + bounds.height / 2;
  const delta_minor_axis = config.delta_minor_axis_sign * 2;
  const delta_dominant_axis = config.delta_dominant_axis_sign * 150;
  const delta_x =
      config.dominant_axis === 'X' ? delta_dominant_axis : delta_minor_axis;
  const delta_y =
      config.dominant_axis === 'Y' ? delta_dominant_axis : delta_minor_axis;

  const initial_scroll_left =
      delta_x > 0 ? 0 : scroller.scrollWidth - scroller.clientWidth;
  const initial_scroll_top =
      delta_y > 0 ? 0 : scroller.scrollHeight - scroller.clientHeight;
  await waitForScrollReset(test, scroller, initial_scroll_left,
                           initial_scroll_top);
  assert_equals(scroller.scrollLeft, initial_scroll_left,
                'initial scroll left');
  assert_equals(scroller.scrollTop, initial_scroll_top, 'initial scroll top');

  // We use tolerances because the final scroll offsets might not
  // match the input deltas exactly. For touch, the initial gesture movement is
  // affected by touch slop (which is platform-dependent), requiring a loose
  // tolerance of 30 on the dominant axis. Wheel scrolls do not have slop,
  // so we use a small tolerance of 1. For the minor axis in the touch test,
  // we also use a small tolerance so that locking is still disallowed.
  const tolerance_minor_axis = 1;
  const tolerance_dominant_axis = (config.input_source === 'touch') ? 30 : 1;
  const tolerance_x = config.dominant_axis === 'X' ? tolerance_dominant_axis :
                                                     tolerance_minor_axis;
  const tolerance_y = config.dominant_axis === 'Y' ? tolerance_dominant_axis :
                                                     tolerance_minor_axis;

  const end_x = start_x - delta_x;
  const end_y = start_y - delta_y;

  const scrollend_promise =
      waitForScrollEndFallbackToDelayWithoutScrollEvent(scroller);

  if (config.input_source === 'touch') {
    await new test_driver.Actions()
        .addPointer('finger', 'touch')
        .pointerMove(Math.round(start_x), Math.round(start_y))
        .pointerDown()
        .addTick()
        .pause(100)
        .pointerMove(Math.round(end_x), Math.round(end_y))
        .addTick()
        .pause(100)  // Pause to avoid fling
        .pointerUp()
        .send();
  } else if (config.input_source === 'wheel') {
    scroller.addEventListener('wheel', (e) => {
      received_wheel_event = e;
    }, {once: true});

    await new test_driver.Actions()
        .addWheel('wheel')
        .scroll(0, 0, Math.round(delta_x), Math.round(delta_y),
                {origin: scroller})
        .send();
  }

  // Wait for scroll to end.
  await scrollend_promise;

  assert_approx_equals(scroller.scrollLeft - initial_scroll_left, delta_x,
                       tolerance_x, 'horizontal scroll offset');
  assert_approx_equals(scroller.scrollTop - initial_scroll_top, delta_y,
                       tolerance_y, 'vertical scroll offset');

  if (config.input_source === 'wheel') {
    // The wheel delta will always be the unlocked value.
    // With scroll-axis-lock: none, the wheel delta should match the scroll
    // delta.
    assert_not_equals(received_wheel_event, null,
                      'Should have received a wheel event');
    assert_equals(received_wheel_event.deltaX, delta_x,
                  'wheel event deltaX should match sent delta');
    assert_equals(received_wheel_event.deltaY, delta_y,
                  'wheel event deltaY should match sent delta');
  }
}
