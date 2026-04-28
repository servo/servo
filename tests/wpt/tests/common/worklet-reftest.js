/**
 * Imports code into a worklet. E.g.
 *
 * importWorklet(CSS.paintWorklet, {url: 'script.js'});
 * importWorklet(CSS.paintWorklet, '(javascript string)');
 *
 * @param {Worklet} worklet
 * @param {(Object|string)} code
 */
function importWorklet(worklet, code) {
    let url;
    if (typeof code === 'object') {
      url = code.url;
    } else {
      const blob = new Blob([code], {type: 'text/javascript'});
      url = URL.createObjectURL(blob);
    }

    return worklet.addModule(url);
}

/** @private */
async function animationFrames(frames) {
  for (let i = 0; i < frames; i++)
    await new Promise(requestAnimationFrame);
}

// This ensures that a commit is accepted and drawn by waiting for a
// composited animation to start.
/** @private */
async function workletPainted() {
  const animation =
    document.body.animate({ opacity: [0, 1] }, { duration: 1 });
  return animation.finished;
}

/**
 * To make sure that we take the snapshot at the right time, we start a
 * composited animation. Once the composited animation runs we know that
 * an updated composited frame has been produced.
 *
 * @param {Worklet} worklet
 * @param {(Object|string)} code
 */
async function importWorkletAndTerminateTestAfterAsyncPaint(worklet, code) {
    if (typeof worklet === 'undefined') {
        takeScreenshot();
        return;
    }

    await importWorklet(worklet, code);
    await workletPainted();
    takeScreenshot();
}
