function waitForFrameTime(ms) {
  return new Promise(resolve => {
      requestAnimationFrame(t0 => {
        (function tick(now) {
          if (now - t0 < ms) {
            requestAnimationFrame(tick);
            return;
          }
          resolve();
        })(t0);
      });
    });
}
