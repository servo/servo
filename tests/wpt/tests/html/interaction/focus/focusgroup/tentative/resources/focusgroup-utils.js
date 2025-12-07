/*
  Methods for testing the focusgroup feature.
*/

// https://w3c.github.io/webdriver/#keyboard-actions
const kArrowLeft = '\uE012';
const kArrowUp = '\uE013';
const kArrowRight = '\uE014';
const kArrowDown = '\uE015';

// Set the focus on target and send the arrow key press event from it.
function focusAndKeyPress(target, key) {
  target.focus();
  return test_driver.send_keys(target, key);
}

function sendArrowKey(key) {
  return new test_driver.Actions().keyDown(key).keyUp(key).send();
}

// Test bidirectional navigation through a list of elements in visual order.
// Tests forward navigation with kArrowRight and backward navigation with kArrowLeft.
// At boundaries, verifies focus does not move (unless wrap is expected).
async function assert_focus_navigation_bidirectional(elements, shouldWrap = false) {
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
