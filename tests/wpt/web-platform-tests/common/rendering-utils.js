"use strict";

/**
 * Waits until we have at least one frame rendered, regardless of the engine.
 *
 * @returns {Promise}
 */
function waitForAtLeastOneFrame() {
  return new Promise(resolve => {
    // Different web engines work slightly different on this area but 1) waiting
    // for two requestAnimationFrames() to happen one after another and 2)
    // adding a step_timeout(0) to guarantee events have finished should be
    // sufficient to ensure at least one frame has been generated anywhere.
    // See https://bugzilla.mozilla.org/show_bug.cgi?id=1785615
    window.requestAnimationFrame(() => {
      window.requestAnimationFrame(() => {
        setTimeout(resolve, 0);
      });
    });
  });
}
