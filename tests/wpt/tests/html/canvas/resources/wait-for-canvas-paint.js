// Include after /common/reftest-wait.js (if used) so that uncaught exceptions
// and unhandled rejections fail the reftest instead of timing out.
if (typeof failOnError === 'function') {
  window.addEventListener('error', failOnError);
  window.addEventListener('unhandledrejection', failOnError);
}

function waitForCanvasPaint(canvas) {
  if (!(canvas instanceof HTMLCanvasElement)) {
    throw new TypeError(
      `waitForCanvasPaint requires an HTMLCanvasElement, got: ${canvas}`
    );
  }
  if (typeof failIfNot === 'function') {
    failIfNot(canvas.requestPaint, 'Missing canvas.requestPaint');
  }
  if (typeof assert_implements === 'function') {
    assert_implements(canvas.requestPaint, 'Missing canvas.requestPaint');
  }
  return new Promise(resolve => {
    canvas.addEventListener('paint', resolve, {once: true});
    canvas.requestPaint();
  });
}
