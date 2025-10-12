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
