// Copyright (C) 2023 Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description:  Checks basic handling of formatToParts, using long, short,narrow and digital styles.
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

// Utils functions
function* zip(a, b) {
  for (let i = 0; i < a.length; ++i) {
    yield [i, a[i], b[i]];
  }
}

function compare(actual, expected, message) {
  assert.sameValue(Array.isArray(expected), true, `${message}: expected is Array`);
  assert.sameValue(Array.isArray(actual), true, `${message}: actual is Array`);
  assert.sameValue(actual.length, expected.length, `${message}: length`);

  for (const [i, actualEntry, expectedEntry] of zip(actual, expected)) {
    // assertions
    assert.sameValue(actualEntry.type, expectedEntry.type, `type for entry ${i}`);
    assert.sameValue(actualEntry.value, expectedEntry.value, `value for entry ${i}`);
    if (expectedEntry.unit) {
      assert.sameValue(actualEntry.unit, expectedEntry.unit, `unit for entry ${i}`);
    }
  }
}
const duration = {
  hours: 7,
  minutes: 8,
  seconds: 9,
  milliseconds: 123,
  microseconds: 456,
  nanoseconds: 789,
};

const style = "narrow";

const df = new Intl.DurationFormat('en', { style });

const expected = partitionDurationFormatPattern(df, duration);

compare(df.formatToParts(duration), expected, `Using style : ${style}`);
