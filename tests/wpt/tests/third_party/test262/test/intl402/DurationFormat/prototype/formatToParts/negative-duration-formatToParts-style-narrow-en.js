// Copyright (C) 2023 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description: >
  Test formatToParts method with negative duration and "narrow" style
locale: [en]
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

function compare(actual, expected, message) {
  assert.sameValue(Array.isArray(expected), true, `${message}: expected is Array`);
  assert.sameValue(Array.isArray(actual), true, `${message}: actual is Array`);
  assert.sameValue(actual.length, expected.length, `${message}: length`);

  for (let i = 0; i < expected.length; ++i) {
    let actualEntry = actual[i];
    let expectedEntry = expected[i];

    assert.sameValue(actualEntry.type, expectedEntry.type, `type for entry ${i}`);
    assert.sameValue(actualEntry.value, expectedEntry.value, `value for entry ${i}`);
    assert.sameValue("unit" in actualEntry, "unit" in expectedEntry, `unit for entry ${i}`);
    if ("unit" in expectedEntry) {
      assert.sameValue(actualEntry.unit, expectedEntry.unit, `unit for entry ${i}`);
    }
  }
}

const style = "narrow";

const duration = {
  years: -1,
  months: -2,
  weeks: -3,
  days: -4,
  hours: -5,
  minutes: -6,
  seconds: -7,
  milliseconds: -123,
  microseconds: -456,
  nanoseconds: -789,
};

const df = new Intl.DurationFormat("en", { style });

const expected = partitionDurationFormatPattern(df, duration);

compare(df.formatToParts(duration), expected, `Using style : ${style}`);
