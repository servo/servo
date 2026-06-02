'use strict'

 // Regression tests for 'Bypass via insertAdjacentText', reported at
  // https://github.com/w3c/trusted-types/issues/133

  // We are trying to assert that scripts do _not_ get executed. We
  // accomplish by having the script under examination containing a
  // postMessage, and to send a second guaranteed-to-execute postMessage
  // so there's a point in time when we're sure the first postMessage
  // must have arrived (if indeed it had been sent).
  //
  // We'll interpret the message data as follows:
  // - includes "block": error (this message should have been blocked by TT)
  // - includes "count": Count these, and later check against expect_count.
  // - includes "done": Unregister the event handler and finish the test.
  // - all else: Reject, as this is probably an error in the test.
  function checkMessage(t, fn, expect_count) {
    return new Promise((resolve, reject) => {
      let count = 0;
      globalThis.addEventListener("message", function handler(e) {
        t.add_cleanup(() => {
          globalThis.removeEventListener("message", handler);
        });
        if (e.data.includes("block")) {
          reject(`'block' received (${e.data})`);
        } else if (e.data.includes("count")) {
          count = count + 1;
        } else if (e.data.includes("done")) {
          globalThis.removeEventListener("message", handler);
          if (expect_count && count != expect_count) {
            reject(
                `'done' received, but unexpected counts: expected ${expect_count} != actual ${count} (${e.data})`);
          } else {
            resolve(e.data);
          }
        } else {
          reject("unexpected message received: " + e.data);
        }
      });
      fn();
      requestAnimationFrame(_ => requestAnimationFrame(_ => postMessage("done", "*")));
    });
  }
