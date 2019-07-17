// Common checks between checkElement() and checkElementWithoutResourceTiming().
function checkElementInternal(entry, expectedUrl, expectedIdentifier, expectedID, beforeRender,
    expectedElement) {
  assert_equals(entry.entryType, 'element');
  assert_equals(entry.url, expectedUrl);
  assert_equals(entry.identifier, expectedIdentifier);
  assert_equals(entry.startTime, 0);
  assert_equals(entry.duration, 0);
  assert_equals(entry.id, expectedID);
  assert_greater_than_equal(entry.renderTime, beforeRender);
  assert_greater_than_equal(performance.now(), entry.renderTime);
  if (expectedElement !== null)
    assert_equals(entry.element, expectedElement);
}

// Checks that this is an ElementTiming entry with url |expectedUrl|. It also
// does a very basic check on |renderTime|: after |beforeRender| and before now().
function checkElement(entry, expectedUrl, expectedIdentifier, expectedID, beforeRender,
    expectedElement) {
  checkElementInternal(entry, expectedUrl, expectedIdentifier, expectedID, beforeRender,
      expectedElement);
  assert_equals(entry.name, 'image-paint');
  const rt_entries = performance.getEntriesByName(expectedUrl, 'resource');
  assert_equals(rt_entries.length, 1);
  assert_equals(rt_entries[0].responseEnd, entry.responseEnd);
}

function checkElementWithoutResourceTiming(entry, expectedUrl, expectedIdentifier,
    expectedID, beforeRender, expectedElement) {
  checkElementInternal(entry, expectedUrl, expectedIdentifier, expectedID, beforeRender,
      expectedElement);
  assert_equals(entry.name, 'image-paint');
  // No associated resource from ResourceTiming, so the responseEnd should be 0.
  assert_equals(entry.responseEnd, 0);
}

// Checks that the rect matches the desired values [left right top bottom].
function checkRect(entry, expected, description="") {
  assert_equals(entry.intersectionRect.left, expected[0],
    'left of rect ' + description);
  assert_equals(entry.intersectionRect.right, expected[1],
    'right of rect ' + description);
  assert_equals(entry.intersectionRect.top, expected[2],
    'top of rect ' + description);
  assert_equals(entry.intersectionRect.bottom, expected[3],
    'bottom of rect ' + description);
}

// Checks that the intrinsic size matches the desired values.
function checkNaturalSize(entry, width, height) {
  assert_equals(entry.naturalWidth, width);
  assert_equals(entry.naturalHeight, height);
}

function checkTextElement(entry, expectedIdentifier, expectedID, beforeRender,
    expectedElement) {
  checkElementInternal(entry, '', expectedIdentifier, expectedID, beforeRender,
      expectedElement);
  assert_equals(entry.name, 'text-paint');
  assert_equals(entry.responseEnd, 0);
}
