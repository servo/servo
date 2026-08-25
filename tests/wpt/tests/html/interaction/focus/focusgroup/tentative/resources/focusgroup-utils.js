/*
  Focusgroup-specific test helpers.

  This file depends on focus-utils.js being loaded first for generic
  primitives (key constants, focusAndKeyPress, sendTabForward, etc.).

  Direction constants (kRight, kLeft, kUp, kDown) map to platform keys
  via DirectionalInputMap.  Override the map for alternative input
  methods (e.g. spatial nav).
*/

const kUp = "up";
const kDown = "down";
const kLeft = "left";
const kRight = "right";

// TODO: Query the platform/user-agent/WebDriver for the correct
// directional input mapping instead of assuming arrow keys.  See
// https://github.com/WebKit/standards-positions/issues/171#issuecomment-4199418777
const DirectionalInputMap = {
  [kUp]:    kArrowUp,
  [kDown]:  kArrowDown,
  [kLeft]:  kArrowLeft,
  [kRight]: kArrowRight,
  home:  kHome,
  end:   kEnd,
};

function keyForDirection(direction) {
  const key = DirectionalInputMap[direction];
  if (!key) {
    throw new Error(`Unknown direction: "${direction}"`);
  }
  return key;
}

async function focusAndSendDirectionalInput(element, direction) {
  return focusAndKeyPress(element, keyForDirection(direction));
}

// Send a directional key without targeting a specific element.  Use this
// for shadow-DOM elements where test_driver.send_keys() cannot reach the
// target inside a shadow root.
async function sendDirectionalKey(direction) {
  return sendKey(keyForDirection(direction));
}

async function focusAndSendHomeInput(element) {
  return focusAndKeyPress(element, keyForDirection("home"));
}

async function focusAndSendEndInput(element) {
  return focusAndKeyPress(element, keyForDirection("end"));
}

// Test forward and backward navigation through a list of elements in
// visual order.  Tests forward navigation with kRight and backward
// navigation with kLeft.  At boundaries, verifies focus does not move
// (unless wrap is expected).
async function assert_directional_navigation_bidirectional(elements, shouldWrap = false) {
  // Test forward navigation.
  for (let i = 0; i < elements.length; i++) {
    await focusAndSendDirectionalInput(elements[i], kRight);
    const nextIndex = shouldWrap ? (i + 1) % elements.length : Math.min(i + 1, elements.length - 1);
    const expectedElement = elements[nextIndex];
    assert_equals(document.activeElement, expectedElement,
      `From ${elements[i].id}, right should move to ${expectedElement.id}`);
  }

  // Test backward navigation.
  for (let i = elements.length - 1; i >= 0; i--) {
    await focusAndSendDirectionalInput(elements[i], kLeft);
    const prevIndex = shouldWrap ? (i - 1 + elements.length) % elements.length : Math.max(i - 1, 0);
    const expectedElement = elements[prevIndex];
    assert_equals(document.activeElement, expectedElement,
      `From ${elements[i].id}, left should move to ${expectedElement.id}`);
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

  elements[0].focus();
  assert_equals(document.activeElement, elements[0],
    `Failed to focus starting element ${elements[0].id}`);

  for (let i = 0; i < elements.length - 1; i++) {
    await navigateFocusBackward();
    assert_equals(document.activeElement, elements[i + 1],
      `Shift+Tab from ${elements[i].id} should move to ${elements[i + 1].id}`);
  }
}

async function assert_directional_input_does_not_move_focus(element) {
  const directions = [kRight, kLeft, kDown, kUp];

  for (const direction of directions) {
    await focusAndSendDirectionalInput(element, direction);
    assert_equals(document.activeElement, element,
      `Direction ${direction} should not move focus from ${element.id}`);
  }
}
