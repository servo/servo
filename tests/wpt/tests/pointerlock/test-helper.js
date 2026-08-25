/* Requests a pointer lock on `element` and waits for the resulting
 * 'pointerlockchange' event.
 * @param {Element} element - The element to lock the pointer to.
 * @param {boolean} provideTransientActivation - If true, a synthetic click is
 *   performed on `element` first to satisfy the user activation requirement.
 * @returns {Event} The pointerlockchange Event object.
 */
async function lockPointerAndWaitForEvent(element, provideTransientActivation) {
  assert_not_equals(element, null, "element can't be null");
  if (provideTransientActivation) {
    // Make a gesture to provide transient activation so that the UA
    // will allow us to acquire the pointer lock.
    await new test_driver.Actions()
      .pointerMove(0, 0, {
        origin: element
      })
      .pointerDown()
      .pointerUp()
      .send();
  }
  var lockEvent = null;
  var lockPromise = new Promise(resolve => {
    document.addEventListener('pointerlockchange', function(e) {
      lockEvent = e;
      resolve();
    }, {
      once: true
    });
  });
  await element.requestPointerLock();
  await lockPromise;
  assert_equals(document.pointerLockElement, element,
    `document.pointerLockElement should be ${element}.`);
  return lockEvent;
}

/* Calls document.exitPointerLock() and waits for the resulting
 * 'pointerlockchange' event. Asserts that document.pointerLockElement is
 * null afterwards.
 * @returns {Event} The pointerlockchange Event object.
 */
async function unlockPointerAndWaitForEvent() {
  var unlockEvent = null;
  var unlockPromise = new Promise(resolve => {
    document.addEventListener('pointerlockchange', function(e) {
      unlockEvent = e;
      resolve();
    }, {
      once: true
    });
  });
  document.exitPointerLock();
  await unlockPromise;
  assert_equals(document.pointerLockElement, null,
    "document.pointerLockElement should be null.");
  return unlockEvent;
}
