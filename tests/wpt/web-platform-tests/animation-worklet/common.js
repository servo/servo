'use strict';

function registerPassthroughAnimator() {
  return runInAnimationWorklet(`
    registerAnimator('passthrough', class {
      animate(currentTime, effect) { effect.localTime = currentTime; }
    });
  `);
}

function runInAnimationWorklet(code) {
  return CSS.animationWorklet.addModule(
    URL.createObjectURL(new Blob([code], {type: 'text/javascript'}))
  );
}

function waitForAnimationFrames(count, callback) {
  function rafCallback() {
    if (count <= 0) {
      callback();
    } else {
      count -= 1;
      window.requestAnimationFrame(rafCallback);
    }
  }
  rafCallback();
};

// Wait for two main thread frames to guarantee that compositor has produced
// at least one frame. Note that this is a Chrome-only concept.
function waitTwoAnimationFrames(callback) {
  waitForAnimationFrames(2, callback);
};
