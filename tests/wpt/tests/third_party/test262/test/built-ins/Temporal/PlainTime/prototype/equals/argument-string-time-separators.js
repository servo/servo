// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.equals
description: Time separator in string argument can vary
features: [Temporal]
---*/

const tests = [
  ["1976-11-18T12:34:56.987654321", "uppercase T"],
  ["1976-11-18t12:34:56.987654321", "lowercase T"],
  ["1976-11-18 12:34:56.987654321", "space between date and time"],
  ["T12:34:56.987654321", "time-only uppercase T"],
  ["t12:34:56.987654321", "time-only lowercase T"],
];

const instance = new Temporal.PlainTime(12, 34, 56, 987, 654, 321);

tests.forEach(([arg, description]) => {
  const result = instance.equals(arg);

  assert.sameValue(
    result,
    true,
    `variant time separators (${description})`
  );
});
