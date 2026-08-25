// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Time separator in string argument can vary
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const tests = [
  ["1970-01-01T00:00+00:00[UTC]", "uppercase T"],
  ["1970-01-01t00:00+00:00[UTC]", "lowercase T"],
  ["1970-01-01 00:00+00:00[UTC]", "space between date and time"],
];

const timeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, timeZone);

tests.forEach(([arg, description]) => {
  const result = instance.since(arg);

  TemporalHelpers.assertDuration(
    result,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    `variant time separators (${description})`
  );
});
