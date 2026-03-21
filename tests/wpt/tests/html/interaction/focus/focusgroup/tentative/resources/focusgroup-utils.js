/*
  Focusgroup-specific test assertion helpers.

  This file depends on focus-utils.js being loaded first for generic
  primitives (key constants, focusAndKeyPress, sendTabForward, etc.).
*/

// Test bidirectional directional (arrow) navigation through a list of elements in visual order.
// Tests forward navigation with kArrowRight and backward navigation with kArrowLeft.
// At boundaries, verifies focus does not move (unless wrap is expected).
async function assert_arrow_navigation_bidirectional(elements, shouldWrap = false) {
  // Test forward navigation.
  for (let i = 0; i < elements.length; i++) {
    await focusAndKeyPress(elements[i], kArrowRight);
    const nextIndex = shouldWrap ? (i + 1) % elements.length : Math.min(i + 1, elements.length - 1);
    const expectedElement = elements[nextIndex];
    assert_equals(document.activeElement, expectedElement,
      `From ${elements[i].id}, right arrow should move to ${expectedElement.id}`);
  }

  // Test backward navigation.
  for (let i = elements.length - 1; i >= 0; i--) {
    await focusAndKeyPress(elements[i], kArrowLeft);
    const prevIndex = shouldWrap ? (i - 1 + elements.length) % elements.length : Math.max(i - 1, 0);
    const expectedElement = elements[prevIndex];
    assert_equals(document.activeElement, expectedElement,
      `From ${elements[i].id}, left arrow should move to ${expectedElement.id}`);
  }
}

// Test Tab navigation through DOM elements. Unlike assert_focus_navigation_forward
// in shadow-dom's focus-utils.js (which takes string paths and requires shadow-dom.js),
// this takes direct element references. Uses sendTabForward (Actions API) to
// avoid calling focus() on document.body, which would blur the active element
// and break focusgroup exit behaviour on key-conflict elements.
async function assert_focusgroup_tab_navigation(elements) {
  if (elements.length === 0) {
    return;
  }

  elements[0].focus();
  assert_equals(document.activeElement, elements[0],
    `Failed to focus starting element ${elements[0].id}`);

  for (let i = 0; i < elements.length - 1; i++) {
    await sendTabForward();
    assert_equals(document.activeElement, elements[i + 1],
      `Tab from ${elements[i].id} should move to ${elements[i + 1].id}`);
  }
}

// Test Shift+Tab navigation through a list of elements in reverse.
// Mirrors assert_focusgroup_tab_navigation but navigates backward.
// Uses navigateFocusBackward (Actions API) for the same reason as above.
async function assert_focusgroup_shift_tab_navigation(elements) {
  if (elements.length === 0) {
    return;
  }

  // Focus the first element to establish starting point.
  elements[0].focus();
  assert_equals(document.activeElement, elements[0],
    `Failed to focus starting element ${elements[0].id}`);

  for (let i = 0; i < elements.length - 1; i++) {
    await navigateFocusBackward();
    assert_equals(document.activeElement, elements[i + 1],
      `Shift+Tab from ${elements[i].id} should move to ${elements[i + 1].id}`);
  }
}

// Assert that arrow keys do not move focus from the given element.
async function assert_arrow_keys_do_not_move_focus(element) {
  const arrows = [kArrowRight, kArrowLeft, kArrowDown, kArrowUp];
  const arrowNames = ['right', 'left', 'down', 'up'];

  for (let i = 0; i < arrows.length; i++) {
    await focusAndKeyPress(element, arrows[i]);
    assert_equals(document.activeElement, element,
      `Arrow ${arrowNames[i]} should not move focus from ${element.id}`);
  }
}
