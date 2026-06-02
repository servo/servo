// removed in https://github.com/whatwg/html/pull/8229
// was never implemented to begin with, so the name should be available.
test(() => {
  assert_equals(CanvasRenderingContext2D.prototype.scrollPathIntoView, undefined);
}, "CanvasRenderingContext2D.scrollPathIntoView method is removed");
