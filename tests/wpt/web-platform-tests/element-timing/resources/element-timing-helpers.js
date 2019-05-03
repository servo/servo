// Common checks between checkElement() and checkElementWithoutResourceTiming().
function checkElementInternal(entry, expectedName, expectedIdentifier, expectedID, beforeRender,
    expectedElement) {
  assert_equals(entry.entryType, 'element');
  assert_equals(entry.name, expectedName);
  assert_equals(entry.identifier, expectedIdentifier);
  assert_equals(entry.duration, 0);
  assert_equals(entry.id, expectedID);
  assert_greater_than_equal(entry.startTime, beforeRender);
  assert_greater_than_equal(performance.now(), entry.startTime);
  if (expectedElement !== null)
    assert_equals(entry.element, expectedElement);
}

// Checks that this is an ElementTiming entry with name |expectedName|. It also
// does a very basic check on |startTime|: after |beforeRender| and before now().
function checkElement(entry, expectedName, expectedIdentifier, expectedID, beforeRender,
    expectedElement) {
  checkElementInternal(entry, expectedName, expectedIdentifier, expectedID, beforeRender,
      expectedElement);
  const rt_entries = performance.getEntriesByName(expectedName, 'resource');
  assert_equals(rt_entries.length, 1);
  assert_equals(rt_entries[0].responseEnd, entry.responseEnd);
}

function checkElementWithoutResourceTiming(entry, expectedName, expectedIdentifier,
    expectedID, beforeRender, expectedElement) {
  checkElementInternal(entry, expectedName, expectedIdentifier, expectedID, beforeRender,
      expectedElement);
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
