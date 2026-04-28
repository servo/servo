// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Negative zero, as an extended year, is rejected
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "-0000000-01-01T00:02Z[UTC]",
  "-0000000-01-01T00:02+00:00[UTC]",
  "-0000000-01-01T00:02:00.000000000+00:00[UTC]",
];

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.from(arg),
    "reject minus zero as extended year"
  );
});
