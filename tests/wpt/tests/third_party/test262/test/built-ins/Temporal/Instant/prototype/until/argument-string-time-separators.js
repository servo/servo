// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Time separator in string argument can vary
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1970-01-01T00:00Z", "uppercase T"],
  ["1970-01-01t00:00Z", "lowercase T"],
  ["1970-01-01 00:00Z", "space between date and time"],
];

const instance = new Temporal.Instant(0n);

tests.forEach(([arg, description]) => {
  const result = instance.until(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `variant time separators (${description})`
  );
});
