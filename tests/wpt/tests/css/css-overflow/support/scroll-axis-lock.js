'use strict';

function test_driver_touch_scroll(start_x, start_y, end_x, end_y) {
  const pointer_down_duration = 100;
  return new test_driver.Actions()
      .addPointer('finger', 'touch')
      .pointerMove(start_x, start_y)
      .pointerDown()
      .addTick()
      .pause(pointer_down_duration)
      .pointerMove(end_x, end_y)
      .addTick()
      .pause(pointer_down_duration)  // Pause to avoid fling
      .pointerUp()
      .send();
}

function test_driver_wheel_scroll(delta_x, delta_y, origin) {
  return new test_driver.Actions()
      .addWheel('wheel')
      .scroll(0, 0, delta_x, delta_y, {origin})
      .send();
}

/**
 * Run assertions to check whether a scroller applies scroll axis lock.
 *
 * @param {test}          The test object that run the assertions.
 * @param {config}        Test configuration
 *   - scroller: Element whose scrolling is tested.
 *   - input_source: the input triggering the scroll ("touch" or "wheel").
 *   - expects_scroll_axis_lock: If true, the test will assert that the scroller
 *     will rail to the dominant axis. If false, it will assert the scroller can
 *     scroll freely on the minor axis too.
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
  const expected_delta_minor_axis =
      config.expects_scroll_axis_lock ? 0 : delta_minor_axis;
  const expected_delta_x = config.dominant_axis === 'X' ?
      delta_dominant_axis :
      expected_delta_minor_axis;
  const expected_delta_y = config.dominant_axis === 'Y' ?
      delta_dominant_axis :
      expected_delta_minor_axis;

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
    await test_driver_touch_scroll(Math.round(start_x), Math.round(start_y),
                                   Math.round(end_x), Math.round(end_y));
  } else if (config.input_source === 'wheel') {
    scroller.addEventListener('wheel', (e) => {
      received_wheel_event = e;
    }, {once: true});

    await test_driver_wheel_scroll(Math.round(delta_x), Math.round(delta_y),
                                   scroller);
  }

  // Wait for scroll to end.
  await scrollend_promise;

  assert_approx_equals(scroller.scrollLeft - initial_scroll_left,
                       expected_delta_x, tolerance_x,
                       'horizontal scroll offset');
  assert_approx_equals(scroller.scrollTop - initial_scroll_top,
                       expected_delta_y, tolerance_y, 'vertical scroll offset');

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

function scroll_is_axis_locked(input_source) {
  // Append a temporary div to make viewport scrollable along both axis.
  const overflowing_top_layer = document.createElement('div');
  overflowing_top_layer.style.position = 'absolute';
  overflowing_top_layer.style.left = '0';
  overflowing_top_layer.style.top = '0';
  overflowing_top_layer.style.width = '500vw';
  overflowing_top_layer.style.height = '500vh';
  overflowing_top_layer.style.zIndex = 1000;
  document.body.appendChild(overflowing_top_layer);

  return (async _ => {
           await waitForCompositorReady();

           // Reset scroll position of viewport if necessary.
           if (document.scrollingElement.scrollLeft != 0 ||
               document.scrollingElement.scrollTop != 0) {
             const scrollend_promise = waitForScrollendEventNoTimeout(document);
             document.scrollingElement.scrollTo(0, 0);
             await scrollend_promise;
           }

           // Perform an almost vertical scroll movement and verify whether the
           // small horizontal component was ignored.
           const scrollend_promise = waitForScrollendEventNoTimeout(document);
           const half_width = Math.floor(window.innerWidth / 2);
           const half_height = Math.floor(window.innerHeight / 2);
           const start_x = half_width;
           const start_y = half_height;
           const delta_x = 2;
           const delta_y = half_height - 1;
           switch (input_source) {
             case 'touch':
               await test_driver_touch_scroll(
                   start_x, start_y, start_x - delta_x, start_y - delta_y);
               break;
             case 'wheel':
               await test_driver_wheel_scroll(delta_x, delta_y, 'viewport');
               break;
             default:
               throw new Error(`Unsupported input source ${input_source}`);
           }
           await scrollend_promise;
           return document.scrollingElement.scrollLeft === 0;
         })()
      .finally(_ => overflowing_top_layer.remove());
}
