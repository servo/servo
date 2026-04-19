// Copyright 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks basic handling of dayPeriod, short format.
features: [Intl.DateTimeFormat-dayPeriod]
includes: [compareArray.js]
---*/

// Each expected dayPeriod value must be a) contiguous, and
// b) represented in sequence.
var expectedDayPeriods = [
  'in the morning',
  'noon',
  'in the afternoon',
  'in the evening',
  'at night'
];

// Cover all 24 hours of a single day.
var inputs = [];
for (var h = 0; h < 24; h++) {
  inputs.push(new Date(2017, 11, 12,  h, 0, 0, 0));
}

var formatter = new Intl.DateTimeFormat('en', {
  dayPeriod: 'short'
});

function assertParts(parts, message) {
  assert.sameValue(parts.length, 1, `length should be 1, ${message}`);
  assert.sameValue(parts[0].type, 'dayPeriod', `part type is dayPeriod. ${message}`);
}

// Verify complete and exclusive representation.
var observedDayPeriods = [];
var unexpectedDayPeriods = [];
for (var h = 0; h < 24; h++) {
  var parts = formatter.formatToParts(inputs[h]);
  assertParts(parts, 'dayPeriod-only formatting for ' + inputs[h]);
  var dayPeriod = parts[0].value;
  observedDayPeriods.push(dayPeriod);
  if (expectedDayPeriods.indexOf(dayPeriod) === -1) {
    unexpectedDayPeriods.push(dayPeriod);
  }
}
var unusedDayPeriods = expectedDayPeriods.filter(function (dayPeriod) {
  return observedDayPeriods.indexOf(dayPeriod) === -1;
});
assert.compareArray(unexpectedDayPeriods, [],
  'unexpected dayPeriods: ' + unexpectedDayPeriods.join());
assert.compareArray(unusedDayPeriods, [],
  'unused dayPeriods: ' + unusedDayPeriods.join());

function arrayAt(arr, relIndex) {
  var realIndex = relIndex < 0 ? arr.length + relIndex : relIndex;
  if (realIndex < 0 || realIndex >= arr.length) return undefined;
  return arr[realIndex];
}

// Verify ordering, accounting for the possibility of one value spanning day
// transitions.
var transitionCount = 0;
for (var h = 0; h < 24; h++) {
  var dayPeriod = observedDayPeriods[h];
  var prevDayPeriod = arrayAt(observedDayPeriods, h - 1);
  if (dayPeriod === prevDayPeriod) continue;
  transitionCount++;
  var i = expectedDayPeriods.indexOf(dayPeriod);
  assert.sameValue(prevDayPeriod, arrayAt(expectedDayPeriods, i - 1),
    dayPeriod + ' must be preceded by ' + prevDayPeriod);
}
assert.sameValue(transitionCount, expectedDayPeriods.length,
  'dayPeriods must be contiguous');

var numericFormatter = new Intl.DateTimeFormat('en', {
  dayPeriod: 'short',
  hour: 'numeric'
});

function assertPartsNumeric(parts, hour, expected, message) {
  assert.sameValue(parts.length, 3, `length should be 3, ${message}`);
  assert.sameValue(parts[0].value, hour, `hour part value. ${message}`);
  assert.sameValue(parts[0].type, 'hour', `hour part type. ${message}`);
  assert.sameValue(parts[1].value, ' ', `literal part value. ${message}`);
  assert.sameValue(parts[1].type, 'literal', `literal part type. ${message}`);
  assert.sameValue(parts[2].value, expected, `expected part value. ${message}`);
  assert.sameValue(parts[2].type, 'dayPeriod', `expected part type. ${message}`);
}

for (var h = 0; h < 24; h++) {
  assertPartsNumeric(
    numericFormatter.formatToParts(inputs[h]),
    // Hour "00" is represented as "12".
    String((h % 12) || 12),
    observedDayPeriods[h],
    'numeric hour must precede dayPeriod'
  );
}
