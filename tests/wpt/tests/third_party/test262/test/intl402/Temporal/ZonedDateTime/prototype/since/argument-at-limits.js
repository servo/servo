// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: One of the operands is at the limit of the supported epoch ns range
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const max = new Temporal.ZonedDateTime(86400_0000_0000_000_000_000n, "UTC");

for (const timeZone of ["Etc/GMT+0", "Europe/Amsterdam", "America/Vancouver"]) {
  const limit = max.withTimeZone(timeZone);
  const instance = new Temporal.PlainDateTime(1970, 1, 1, 1, 1, 1, 1, 1, 1).toZonedDateTime(timeZone);

  instance.since(limit, { largestUnit: "years" });  // should not throw
  limit.since(instance, { largestUnit: "years" });  // should not throw

  const resultTimeUnit = instance.since(limit, { largestUnit: "seconds" });
  TemporalHelpers.assertDurationsEqual(
    limit.since(instance, { largestUnit: "seconds" }),
    resultTimeUnit.negated(),
    `Arithmetic since limit with time largestUnit is self-consistent (${timeZone})`
  );
}

const min = new Temporal.ZonedDateTime(-86400_0000_0000_000_000_000n, "UTC");

for (const timeZone of ["Etc/GMT+0", "Europe/Amsterdam", "America/Vancouver"]) {
  const limit = min.withTimeZone(timeZone);
  const instance = new Temporal.PlainDateTime(1970, 9, 1, 15, 47, 32).toZonedDateTime(timeZone);

  instance.since(limit, { largestUnit: "years" });  // should not throw
  limit.since(instance, { largestUnit: "years" });  // should not throw

  const resultTimeUnit = instance.since(limit, { largestUnit: "seconds" });
  TemporalHelpers.assertDurationsEqual(
    limit.since(instance, { largestUnit: "seconds" }),
    resultTimeUnit.negated(),
    `Arithmetic since limit with time largestUnit is self-consistent (${timeZone})`
  );
}
