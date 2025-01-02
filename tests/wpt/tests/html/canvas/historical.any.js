// removed in https://github.com/whatwg/html/pull/9979
test(() => {
  assert_equals(OffscreenCanvasRenderingContext2D.prototype.commit, undefined);
}, "OffscreenCanvasRenderingContext2D.commit method is removed");
