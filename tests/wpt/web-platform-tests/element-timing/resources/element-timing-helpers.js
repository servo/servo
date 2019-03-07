// Checks that this is an ElementTiming entry with name |expectedName|. It also
// does a very basic check on |startTime|: after |beforeRender| and before now().
function checkElement(entry, expectedName, expectedIdentifier, beforeRender) {
  assert_equals(entry.entryType, 'element');
  assert_equals(entry.name, expectedName);
  assert_equals(entry.identifier, expectedIdentifier);
  assert_equals(entry.duration, 0);
  assert_greater_than_equal(entry.startTime, beforeRender);
  assert_greater_than_equal(performance.now(), entry.startTime);
  const rt_entries = performance.getEntriesByName(expectedName, 'resource');
  assert_equals(rt_entries.length, 1);
  assert_equals(rt_entries[0].responseEnd, entry.responseEnd);
}

// Checks that the rect matches the desired values [left right top bottom]
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
