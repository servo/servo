
// Logs (appends) an HTML string to a logger element in a list format.
// An element in the document with id "logger" will be used as the log
// container.
function log(str) {
  const entry = document.createElement('li');
  entry.innerHTML = str;
  const loggerElement = document.getElementById('logger');
  loggerElement.appendChild(entry);
  return entry;
}

// Common setup for window management tests. Performs some basic assertions, and
// then waits for a click on the `setUpButton` element (for manual tests).
// Example usage:
//  promise_test(async setUpTest => {
//    await setUpWindowManagement(setUpTest, setUpButton);
//    ...
//  });
async function setUpWindowManagement(setUpTest, setUpButton) {
  assert_true(
    'getScreenDetails' in self && 'isExtended' in screen,
    `API not supported; use Chromium (not content_shell) and enable
     chrome://flags/#enable-experimental-web-platform-features`);
  if (!screen.isExtended)
    log(`WARNING: Use multiple screens for full test coverage`);
  if (window.location.href.startsWith('file'))
    log(`WARNING: Run via 'wpt serve'; file URLs lack permission support`);

  try {  // Support manual testing where test_driver is not running.
    await test_driver.set_permission({ name: 'window-placement' }, 'granted');
  } catch {
  }
  const setUpWatcher = new EventWatcher(setUpTest, setUpButton, ['click']);
  const setUpClick = setUpWatcher.wait_for('click');
  try {  // Support manual testing where test_driver is not running.
    await test_driver.click(setUpButton);
  } catch {
  }
  await setUpClick;
  setUpButton.disabled = true;
}


// Adds a button to the given `buttonContainer` element with the contents of
// `name`. Attaches an event watcher to the given test and waits for a signal
// from the test driver to click the button. If no test driver is available
// (manual testing) then awaits an actual click from the user instead. If
// `disableOnClick` is true, the button will also be disabled after it is
// clicked.
async function addTestTriggerButtonAndAwaitClick(buttonContainer, name, test) {
  const button = document.createElement('button');
  button.innerHTML = name;
  const entry = document.createElement('li');
  entry.appendChild(button);
  buttonContainer.appendChild(entry);
  const testWatcher = new EventWatcher(test, button, ['click']);
  const buttonClick = testWatcher.wait_for('click');
  // Disable the button when it is clicked.
  button.onclick = function() {
    button.disabled = true;
  };
  try {  // Support manual testing where test_driver is not running.
    await test_driver.click(button);
  } catch {
  }
  await buttonClick;
}
