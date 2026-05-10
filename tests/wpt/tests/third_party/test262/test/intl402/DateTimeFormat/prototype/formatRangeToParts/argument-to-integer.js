// Copyright (C) 2017 André Bargull. All rights reserved.
// Copyright (C) 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  TimeClip applies ToInteger on its input value.
info: |
  Intl.DateTimeFormat.prototype.formatRangeToParts ( startDate , endDate )

  5. Let x be ? ToNumber(startDate).
  6. Let y be ? ToNumber(endDate).

  TimeClip ( time )
  ...
  3. Let clippedTime be ! ToInteger(time).
  4. If clippedTime is -0, set clippedTime to +0.
  5. Return clippedTime.
features: [Intl.DateTimeFormat-formatRange]
---*/

function* zip(a, b) {
  assert.sameValue(a.length, b.length);
  for (let i = 0; i < a.length; ++i) {
    yield [i, a[i], b[i]];
  }
}

function compare(actual, expected, message) {
  for (const [i, actualEntry, expectedEntry] of zip(actual, expected)) {
    assert.sameValue(actualEntry.type, expectedEntry.type, `${message}: type for entry ${i}`);
    assert.sameValue(actualEntry.value, expectedEntry.value, `${message}: value for entry ${i}`);
    assert.sameValue(actualEntry.source, expectedEntry.source, `${message}: source for entry ${i}`);
  }
}

// Switch to a time format instead of using DateTimeFormat's default date-only format.
const dtf = new Intl.DateTimeFormat(undefined, {
    hour: "numeric", minute: "numeric", second: "numeric"
});
const date = Date.now();
const expected = dtf.formatRangeToParts(0, date);

compare(dtf.formatRangeToParts(-0.9, date), expected, "formatRangeToParts(-0.9)");
compare(dtf.formatRangeToParts(-0.5, date), expected, "formatRangeToParts(-0.5)");
compare(dtf.formatRangeToParts(-0.1, date), expected, "formatRangeToParts(-0.1)");
compare(dtf.formatRangeToParts(-Number.MIN_VALUE, date), expected, "formatRangeToParts(-Number.MIN_VALUE)");
compare(dtf.formatRangeToParts(-0, date), expected, "formatRangeToParts(-0)");
compare(dtf.formatRangeToParts(+0, date), expected, "formatRangeToParts(+0)");
compare(dtf.formatRangeToParts(Number.MIN_VALUE, date), expected, "formatRangeToParts(Number.MIN_VALUE)");
compare(dtf.formatRangeToParts(0.1, date), expected, "formatRangeToParts(0.1)");
compare(dtf.formatRangeToParts(0.5, date), expected, "formatRangeToParts(0.5)");
compare(dtf.formatRangeToParts(0.9, date), expected, "formatRangeToParts(0.9)");
