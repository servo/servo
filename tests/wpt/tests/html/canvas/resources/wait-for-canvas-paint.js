function waitForCanvasPaint(canvas) {
  if (!(canvas instanceof HTMLCanvasElement)) {
    throw new TypeError(
      `waitForCanvasPaint requires an HTMLCanvasElement, got: ${canvas}`
    );
  }
  return new Promise(resolve => {
    canvas.addEventListener('paint', resolve, {once: true});
    canvas.requestPaint();
  });
}
