// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  Test format method with negative zero as the input.
locale: [en-US]
includes: [testIntl.js]
features: [Intl.DurationFormat]
---*/

const units = [
  "years",
  "months",
  "weeks",
  "days",
  "hours",
  "minutes",
  "seconds",
  "milliseconds",
  "microseconds",
  "nanoseconds",
];

for (let unit of units) {
  let positiveZero = {
    [unit]: +0,
  };

  let negativeZero = {
    [unit]: -0,
  };

  let auto = new Intl.DurationFormat("en", {[unit + "Display"]: "auto"});

  assert.sameValue(
    auto.format(positiveZero),
    "",
    `+0 ${unit} is the empty string when display is "auto"`
  );

  assert.sameValue(
    auto.format(negativeZero),
    "",
    `-0 ${unit} is the empty string when display is "auto"`
  );

  let always = new Intl.DurationFormat("en", {[unit + "Display"]: "always"});

  let expected = formatDurationFormatPattern(always, positiveZero);

  assert.sameValue(
    always.format(negativeZero),
    expected,
    `-0 ${unit} produces the same output as +0 ${unit} when display is "always"`
  );
}
