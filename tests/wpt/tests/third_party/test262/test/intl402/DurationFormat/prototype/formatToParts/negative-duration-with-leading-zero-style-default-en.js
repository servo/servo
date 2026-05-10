// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.formatToParts
description: >
  Test formatToParts method with negative duration and leading zero using the default style.
locale: [en-US]
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

const duration = {
  hours: 0,
  seconds: -1,
};

const df = new Intl.DurationFormat("en", {hoursDisplay: "always"});

const expected = partitionDurationFormatPattern(df, duration);

compare(df.formatToParts(duration), expected, `Using style : default`);
