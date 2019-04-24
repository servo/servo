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

// TODO(majidvp): This is used to sidestep a bug where we currently animate
// with currentTime=NaN when scroll timeline is not active. Remove once we fix
// http://crbug.com/937456
function registerPassthroughExceptNaNAnimator() {
  return runInAnimationWorklet(`
    registerAnimator('passthrough_except_nan', class {
      animate(currentTime, effect) {
        if (Number.isNaN(currentTime)) return;
        effect.localTime = currentTime;
      }
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

async function waitForAnimationFrameWithCondition(condition) {
  do {
    await new Promise(window.requestAnimationFrame);
  } while (!condition())
}

async function waitForDocumentTimelineAdvance() {
  const timeAtStart = document.timeline.currentTime;
  do {
    await new Promise(window.requestAnimationFrame);
  } while (timeAtStart === document.timeline.currentTime)
}

// Wait until animation's effect has a non-null localTime.
async function waitForNotNullLocalTime(animation) {
  await waitForAnimationFrameWithCondition(_ => {
    return animation.effect.getComputedTiming().localTime !== null;
  });
}