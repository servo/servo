// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
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

const instance = new Temporal.PlainDate(2000, 5, 2);

tests.forEach(([arg, description]) => {
  const result = instance.toZonedDateTime({ plainTime: arg, timeZone: "UTC" });

  assert.sameValue(
    result.epochNanoseconds,
    957_270_896_987_654_321n,
    `variant time separators (${description})`
  );
});
