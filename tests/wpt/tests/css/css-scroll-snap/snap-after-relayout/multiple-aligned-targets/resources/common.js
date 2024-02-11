// Utility functions for scroll snap tests which verify User-Agents' snap point
// selection logic when multiple snap targets are aligned.
// It depends on methods in /resources/testdriver-actions.js and
// /dom/event/scrolling/scroll_support.js so html files using these functions
// should include those files as <script>s.

// This function should be used by scroll snap WPTs wanting to test snap target
// selection when scrolling to multiple aligned targets.
// It assumes scroll-snap-align: start alignment and tries to align to the list
// of snap targets provided, |elements|, which are all expected to be at the
// same offset.
async function scrollToAlignedElementsInAxis(scroller, elements, axis) {
  let target_offset_y = null;
  let target_offset_x = null;
  if (axis == "y") {
    for (const e of elements) {
      if (target_offset_y) {
        assert_equals(e.offsetTop, target_offset_y,
          `${e.id} is at y offset ${target_offset_y}`);
      } else {
        target_offset_y = e.offsetTop;
      }
    }
    assert_equals();
  } else {
    for (const e of elements) {
      if (target_offset_x) {
        assert_equals(e.offsetLeft, target_offset_x,
          `${e.id} is at x offset ${target_offset_x}`);
      } else {
        target_offset_x = e.offsetLeft;
      }
    }
  }
  assert_not_equals(target_offset_x || target_offset_y, null);

  const scrollend_promise = waitForScrollendEventNoTimeout(scroller);
  await new test_driver.Actions().scroll(0, 0,
    (target_offset_x || 0) - scroller.scrollLeft,
    (target_offset_y || 0) - scroller.scrollTop,
    { origin: scroller })
    .send();
  await scrollend_promise;
  if (axis == "y") {
    assert_equals(scroller.scrollTop, target_offset_y, "vertical scroll done");
  } else {
    assert_equals(scroller.scrollLeft,target_offset_x, "horizontal scroll done");
  }
}

// This function verifies the snap target that a scroller picked by triggerring
// a layout change and observing which target is followed. Tests using this
// method should ensure that there is at least 100px of room to scroll in the
// desired axis.
// It assumes scroll-snap-align: start alignment.
function verifySelectedSnapTarget(scroller, expected_snap_target, axis) {
  // Save initial style.
  const initial_left = getComputedStyle(expected_snap_target).left;
  const initial_top = getComputedStyle(expected_snap_target).top;
  if (axis == "y") {
    // Move the expected snap target along the y axis.
    const initial_scroll_top = scroller.scrollTop;
    const target_top = expected_snap_target.offsetTop + 100;
    expected_snap_target.style.top = `${target_top}px`;
    assert_equals(scroller.scrollTop, expected_snap_target.offsetTop,
      `scroller followed ${expected_snap_target.id} after layout change`);
    assert_not_equals(scroller.scrollTop, initial_scroll_top,
      "scroller actually scrolled in y axis");
  } else {
    // Move the expected snap target along the y axis.
    const initial_scroll_left = scroller.scrollLeft;
    const target_left = expected_snap_target.offsetLeft + 100;
    expected_snap_target.style.left = `${target_left}px`;
    assert_equals(scroller.scrollLeft, expected_snap_target.offsetLeft,
      `scroller followed ${expected_snap_target.id} after layout change`);
    assert_not_equals(scroller.scrollLeft, initial_scroll_left,
      "scroller actually scrolled in x axis");
  }
  // Undo style changes.
  expected_snap_target.style.top = initial_top;
  expected_snap_target.style.left = initial_left;
}

// This is a utility function for tests which verify that the correct element
// is snapped to when snapping at the end of a scroll.
async function runScrollSnapSelectionVerificationTest(t, scroller, aligned_elements,
                                                expected_target, axis) {
  // Save initial scroll offset.
  const initial_scroll_left = scroller.scrollLeft;
  const initial_scroll_top = scroller.scrollTop;
  await scrollToAlignedElementsInAxis(scroller, aligned_elements, axis);
  verifySelectedSnapTarget(scroller, expected_target, axis);
  // Restore initial scroll offsets.
  const scrollend_promise = new Promise((resolve) => {
    scroller.addEventListener("scrollend", resolve);
  });
  scroller.scrollTo(initial_scroll_left, initial_scroll_top);
  await scrollend_promise;
}

// This is a utility function for tests verifying that a layout shift does not
// cause a scroller to change its selected snap target.
// It assumes the element to be aligned have scroll-snap-align: start.
// It tries to align the list of snap targets provided, |elements| with the
// current snap target.
function shiftLayoutToAlignElements(elements, target, axis) {
  for (let element of elements) {
    if (axis == "y") {
      element.style.top = `${target.offsetTop}px`;
    } else {
      element.style.left = `${target.offsetLeft}px`;
    }
  }
}

// This is a utility function for tests verifying that a layout shift does not
// cause a scroller to change its selected snap target.
// It assumes scroll-snap-align: start alignment.
async function runLayoutSnapSeletionVerificationTest(t, scroller, elements_to_align,
  expected_target, axis) {
  // Save initial scroll offsets and position.
  const initial_scroll_left = scroller.scrollLeft;
  const initial_scroll_top = scroller.scrollTop;
  let initial_tops = [];
  for (const element of elements_to_align) {
    initial_tops.push(getComputedStyle(element).top);
  }

  shiftLayoutToAlignElements(elements_to_align, expected_target, axis);
  verifySelectedSnapTarget(scroller, expected_target, axis);

  // Restore initial scroll offset and position states.
  let num_elements = initial_tops.length;
  for (let i = 0; i < num_elements; i++) {
    elements_to_align[i].style.top = initial_tops[i];
  }
  // Restore initial scroll offsets.
  const scrollend_promise = new Promise((resolve) => {
    scroller.addEventListener("scrollend", resolve);
  });
  scroller.scrollTo(initial_scroll_left, initial_scroll_top);
  await scrollend_promise;
}

function focusAndAssert(element, preventScroll=false) {
  element.focus({preventScroll: preventScroll});
  assert_equals(document.activeElement, element);
}
