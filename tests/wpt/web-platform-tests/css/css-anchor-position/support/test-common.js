// Asserts that the anchored element is at the top/bottom/left/right of the
// anchor.
function assert_fallback_position(anchored, anchor, direction) {
  let anchoredRect = anchored.getBoundingClientRect();
  let anchorRect = anchor.getBoundingClientRect();
  let message = `Anchored element should be at the ${direction} of anchor`;
  switch (direction) {
    case 'top':
      assert_equals(anchoredRect.bottom, anchorRect.top, message);
      return;
    case 'bottom':
      assert_equals(anchoredRect.top, anchorRect.bottom, message);
      return;
    case 'left':
      assert_equals(anchoredRect.right, anchorRect.left, message);
      return;
    case 'right':
      assert_equals(anchoredRect.left, anchorRect.right, message);
      return;
    default:
      assert_unreached('unsupported direction');
  }
}

async function waitUntilNextAnimationFrame() {
  return new Promise(resolve => requestAnimationFrame(resolve));
}
