'use strict';

function waitForEvent(target, type, options) {
  return new Promise((resolve, reject) => {
    target.addEventListener(type, resolve, options);
  });
}

function waitForLoad(target) {
  return waitForEvent(target, 'load');
}

function timeOut(test, ms) {
  return new Promise((resolve, reject) => {
    test.step_timeout(resolve, ms);
  });
}

// If an element with autofocus is connected to a document and this function
// is called, the autofocus result is deterministic after returning from the
// function.
// Exception: If the document has script-blocking style sheets, this function
// doesn't work well.
async function waitUntilStableAutofocusState(test) {
  // TODO: Update this for https://github.com/web-platform-tests/wpt/pull/17929
  await timeOut(test, 100);
}
