// Here's how waitForNotification works:
//
// - myTestFunction0()
//   - waitForNotification(myTestFunction1)
//     - requestAnimationFrame()
//   - Modify DOM in a way that should trigger an IntersectionObserver callback.
// - BeginFrame
//   - requestAnimationFrame handler runs
//     - First step_timeout()
//   - Style, layout, paint
//   - IntersectionObserver generates new notifications
//     - Posts a task to deliver notifications
// - First step_timeout handler runs
//   - Second step_timeout()
// - Task to deliver IntersectionObserver notifications runs
//   - IntersectionObserver callbacks run
// - Second step_timeout handler runs
//   - myTestFunction1()
//     - [optional] waitForNotification(myTestFunction2)
//       - requestAnimationFrame()
//     - Verify newly-arrived IntersectionObserver notifications
//     - [optional] Modify DOM to trigger new notifications
function waitForNotification(t, f) {
  requestAnimationFrame(function() {
    t.step_timeout(function() { t.step_timeout(f); });
  });
}

// The timing of when runTestCycle is called is important.  It should be
// called:
//
//   - Before or during the window load event, or
//   - Inside of a prior runTestCycle callback, *before* any assert_* methods
//     are called.
//
// Following these rules will ensure that the test suite will not abort before
// all test steps have run.
function runTestCycle(f, description) {
  async_test(function(t) {
    waitForNotification(t, t.step_func_done(f));
  }, description);
}

// Root bounds for a root with an overflow clip as defined by:
//   http://wicg.github.io/IntersectionObserver/#intersectionobserver-root-intersection-rectangle
function contentBounds(root) {
  var left = root.offsetLeft + root.clientLeft;
  var right = left + root.clientWidth;
  var top = root.offsetTop + root.clientTop;
  var bottom = top + root.clientHeight;
  return [left, right, top, bottom];
}

// Root bounds for a root without an overflow clip as defined by:
//   http://wicg.github.io/IntersectionObserver/#intersectionobserver-root-intersection-rectangle
function borderBoxBounds(root) {
  var left = root.offsetLeft;
  var right = left + root.offsetWidth;
  var top = root.offsetTop;
  var bottom = top + root.offsetHeight;
  return [left, right, top, bottom];
}

function clientBounds(element) {
  var rect = element.getBoundingClientRect();
  return [rect.left, rect.right, rect.top, rect.bottom];
}

function rectArea(rect) {
  return (rect.left - rect.right) * (rect.bottom - rect.top);
}

function checkRect(actual, expected, description, all) {
  if (!expected.length)
    return;
  assert_equals(actual.left | 0, expected[0] | 0, description + '.left');
  assert_equals(actual.right | 0, expected[1] | 0, description + '.right');
  assert_equals(actual.top | 0, expected[2] | 0, description + '.top');
  assert_equals(actual.bottom | 0, expected[3] | 0, description + '.bottom');
}

function checkLastEntry(entries, i, expected) {
  assert_equals(entries.length, i + 1, 'entries.length');
  if (expected) {
    checkRect(
        entries[i].boundingClientRect, expected.slice(0, 4),
        'entries[' + i + '].boundingClientRect', entries[i]);
    checkRect(
        entries[i].intersectionRect, expected.slice(4, 8),
        'entries[' + i + '].intersectionRect', entries[i]);
    checkRect(
        entries[i].rootBounds, expected.slice(8, 12),
        'entries[' + i + '].rootBounds', entries[i]);
    if (expected.length > 12) {
      assert_equals(
          entries[i].isIntersecting, expected[12],
          'entries[' + i + '].isIntersecting');
    }
  }
}

function checkJsonEntry(actual, expected) {
  checkRect(
      actual.boundingClientRect, expected.boundingClientRect,
      'entry.boundingClientRect');
  checkRect(
      actual.intersectionRect, expected.intersectionRect,
      'entry.intersectionRect');
  if (actual.rootBounds == 'null')
    assert_equals(expected.rootBounds, 'null', 'rootBounds is null');
  else
    checkRect(actual.rootBounds, expected.rootBounds, 'entry.rootBounds');
  assert_equals(actual.target, expected.target);
}

function checkJsonEntries(actual, expected, description) {
  test(function() {
    assert_equals(actual.length, expected.length);
    for (var i = 0; i < actual.length; i++)
      checkJsonEntry(actual[i], expected[i]);
  }, description);
}

function checkIsIntersecting(entries, i, expected) {
  assert_equals(entries[i].isIntersecting, expected,
    'entries[' + i + '].target.isIntersecting equals ' + expected);
}