'use strict';

function registerPassthroughAnimator() {
  return runInAnimationWorklet(`
    registerAnimator('passthrough', class {
      animate(currentTime, effect) { effect.localTime = currentTime; }
    });
  `);
}

function registerConstantLocalTimeAnimator(localTime) {
  return runInAnimationWorklet(`
    registerAnimator('constant_time', class {
      animate(currentTime, effect) { effect.localTime = ${localTime}; }
    });
  `);
}


function runInAnimationWorklet(code) {
  return CSS.animationWorklet.addModule(
    URL.createObjectURL(new Blob([code], {type: 'text/javascript'}))
  );
}

function waitForAsyncAnimationFrames(count) {
  // In Chrome, waiting for N+1 main thread frames guarantees that compositor has produced
  // at least N frames.
  // TODO(majidvp): re-evaluate this choice once other browsers have implemented
  // AnimationWorklet.
  return waitForAnimationFrames(count + 1);
}
