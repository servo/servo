// Copyright (C) 2017 André Bargull. All rights reserved.
// Copyright (C) 2019 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: >
  TimeClip applies ToInteger on its input value.
info: |
  Intl.DateTimeFormat.prototype.formatRange ( startDate , endDate )

  5. Let x be ? ToNumber(startDate).
  6. Let y be ? ToNumber(endDate).

  TimeClip ( time )
  ...
  3. Let clippedTime be ! ToInteger(time).
  4. If clippedTime is -0, set clippedTime to +0.
  5. Return clippedTime.
features: [Intl.DateTimeFormat-formatRange]
---*/

// Switch to a time format instead of using DateTimeFormat's default date-only format.
const dtf = new Intl.DateTimeFormat(undefined, {
    hour: "numeric", minute: "numeric", second: "numeric"
});
const date = Date.now();
const expected = dtf.formatRange(0, date);

assert.sameValue(dtf.formatRange(-0.9, date), expected, "formatRange(-0.9)");
assert.sameValue(dtf.formatRange(-0.5, date), expected, "formatRange(-0.5)");
assert.sameValue(dtf.formatRange(-0.1, date), expected, "formatRange(-0.1)");
assert.sameValue(dtf.formatRange(-Number.MIN_VALUE, date), expected, "formatRange(-Number.MIN_VALUE)");
assert.sameValue(dtf.formatRange(-0, date), expected, "formatRange(-0)");
assert.sameValue(dtf.formatRange(+0, date), expected, "formatRange(+0)");
assert.sameValue(dtf.formatRange(Number.MIN_VALUE, date), expected, "formatRange(Number.MIN_VALUE)");
assert.sameValue(dtf.formatRange(0.1, date), expected, "formatRange(0.1)");
assert.sameValue(dtf.formatRange(0.5, date), expected, "formatRange(0.5)");
assert.sameValue(dtf.formatRange(0.9, date), expected, "formatRange(0.9)");
