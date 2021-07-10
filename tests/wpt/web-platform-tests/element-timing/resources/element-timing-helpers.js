// Common checks between checkElement() and checkElementWithoutResourceTiming().
function checkElementInternal(entry, expectedUrl, expectedIdentifier, expectedID, beforeRender,
    expectedElement) {
  assert_equals(entry.entryType, 'element', 'entryType does not match');
  assert_equals(entry.url, expectedUrl, 'url does not match');
  assert_equals(entry.identifier, expectedIdentifier, 'identifier does not match');
  if (beforeRender != 0) {
    // In this case, renderTime is not 0.
    assert_greater_than(entry.renderTime, 0, 'renderTime should be nonzero');
    assert_equals(entry.startTime, entry.renderTime, 'startTime should equal renderTime');
  } else {
    // In this case, renderTime is 0, so compare to loadTime.
    assert_equals(entry.renderTime, 0, 'renderTime should be zero');
    assert_equals(entry.startTime, entry.loadTime, 'startTime should equal loadTime');
  }
  assert_equals(entry.duration, 0, 'duration should be 0');
  assert_equals(entry.id, expectedID, 'id does not match');
  assert_greater_than_equal(entry.renderTime, beforeRender, 'renderTime greater than beforeRender');
  assert_greater_than_equal(performance.now(), entry.renderTime, 'renderTime bounded by now()');
  if (expectedElement !== null) {
    assert_equals(entry.element, expectedElement, 'element does not match');
    assert_equals(entry.identifier, expectedElement.elementTiming,
        'identifier must be the elementtiming of the element');
    assert_equals(entry.id, expectedElement.id, 'id must be the id of the element');
  }
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
  assert_greater_than_equal(entry.loadTime, rt_entries[0].responseEnd,
    'Image loadTime is after the resource responseEnd');
}

function checkElementWithoutResourceTiming(entry, expectedUrl, expectedIdentifier,
    expectedID, beforeRender, expectedElement) {
  checkElementInternal(entry, expectedUrl, expectedIdentifier, expectedID, beforeRender,
      expectedElement);
  assert_equals(entry.name, 'image-paint');
  // No associated resource from ResourceTiming, so not much to compare loadTime with.
  assert_greater_than(entry.loadTime, 0);
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
  assert_equals(entry.loadTime, 0);
}
