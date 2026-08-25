// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Negative zero, as an extended year, fails
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(0n, "UTC");
const invalidStrings = [
  "-0000000-01-01T00:02Z[UTC]",
  "-0000000-01-01T00:02+00:00[UTC]",
  "-0000000-01-01T00:02:00.000000000+00:00[UTC]",
];

invalidStrings.forEach((arg) => {
  assert.throws(RangeError,
    () => Temporal.ZonedDateTime.compare(arg, instance),
    "cannot use negative zero as extended year (first argument)"
  );
  assert.throws(RangeError,
    () => Temporal.ZonedDateTime.compare(instance, arg),
    "cannot use negative zero as extended year (second argument)"
  );
});
