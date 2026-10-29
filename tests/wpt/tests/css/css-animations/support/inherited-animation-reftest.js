// The first frame commits the parent's animation to the compositor. Blocking
// the main thread in every frame after that, including the one the screenshot
// is taken from, lets the compositor keep advancing the parent while the child
// keeps the value it inherited on the main thread. 20ms is longer than a
// frame at 60Hz (16.7ms), so the main thread falls behind the compositor.
onload = () => {
  requestAnimationFrame(() => {
    requestAnimationFrame(function hold() {
      for (const end = performance.now() + 20; performance.now() < end;) {
      }
      requestAnimationFrame(hold);
    });
  });
};
