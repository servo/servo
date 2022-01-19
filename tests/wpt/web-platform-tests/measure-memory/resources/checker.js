function checkContainer(actual, expected) {
  if (!actual) return true;
  if (!expected) return false;
  return actual.id == expected.id && actual.src == expected.src;
}

function checkAttribution(attribution, expected) {
  assert_own_property(attribution, 'url');
  assert_own_property(attribution, 'scope');
  let found = false;
  for (const e of expected) {
    if (attribution.url === e.url &&
        attribution.scope === e.scope &&
        checkContainer(attribution.container, e.container)) {
      found = true;
      e.found = true;
    }
  }
  assert_true(found, JSON.stringify(attribution) +
      ' is not found in ' + JSON.stringify(expected) + '.');
}

function checkBreakdown(breakdown, expected) {
  assert_own_property(breakdown, 'bytes');
  assert_greater_than_equal(breakdown.bytes, 0);
  assert_own_property(breakdown, 'types');
  for (const memoryType of breakdown.types) {
    assert_equals(typeof memoryType, 'string');
  }
  assert_own_property(breakdown, 'attribution');
  for (const attribution of breakdown.attribution) {
    checkAttribution(attribution, expected);
  }
}

function isEmptyBreakdownEntry(entry) {
  return entry.bytes === 0 && entry.attribution.length === 0 &&
         entry.types.length === 0;
}

function checkMeasureMemory(result, expected) {
  assert_own_property(result, 'bytes');
  assert_own_property(result, 'breakdown');
  let bytes = 0;
  for (let breakdown of result.breakdown) {
    checkBreakdown(breakdown, expected);
    bytes += breakdown.bytes;
  }
  assert_equals(bytes, result.bytes);
  for (const e of expected) {
    if (e.required) {
      assert_true(e.found,
          JSON.stringify(e) + ' did not appear in the result.');
    }
  }
  assert_true(result.breakdown.some(isEmptyBreakdownEntry),
      'The result must include an empty breakdown entry.');
}