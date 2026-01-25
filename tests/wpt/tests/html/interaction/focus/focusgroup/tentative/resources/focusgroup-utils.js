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

function waitForRender() {
  return new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve)));
}

async function navigateFocusForward() {
  await waitForRender();
  const kTab = '\uE004';
  await new test_driver.send_keys(document.body, kTab);
  await waitForRender();
}

async function navigateFocusBackward() {
  await waitForRender();
  const kShift = '\uE008';
  const kTab = '\uE004';
  await new test_driver.Actions()
    .keyDown(kShift)
    .keyDown(kTab)
    .keyUp(kTab)
    .keyUp(kShift)
    .send();
  await waitForRender();
}

// Test sequential (Tab) navigation through a list of elements.
async function assert_focus_navigation_forward(elements) {
  if (elements.length === 0) {
    return;
  }

  // Focus the first element to establish starting point.
  elements[0].focus();
  assert_equals(document.activeElement, elements[0],
    `Failed to focus starting element ${elements[0].id}`);

  for (let i = 0; i < elements.length - 1; i++) {
    await navigateFocusForward();
    assert_equals(document.activeElement, elements[i + 1],
      `Tab from ${elements[i].id} should move to ${elements[i + 1].id}`);
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
