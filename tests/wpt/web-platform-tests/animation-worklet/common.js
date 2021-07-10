'use strict';

function registerPassthroughAnimator() {
  return runInAnimationWorklet(`
    registerAnimator('passthrough', class {
      animate(currentTime, effect) {
        effect.localTime = currentTime;
      }
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

function approxEquals(actual, expected){
  // precision in ms
  const epsilon = 0.005;
  const lowerBound = (expected - epsilon) < actual;
  const upperBound = (expected + epsilon) > actual;
  return lowerBound && upperBound;
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