// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Negative zero, as an extended year, is rejected
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "-000000-10-31T17:45Z",
  "-000000-10-31T17:45+00:00[UTC]",
];
invalidStrings.forEach((timeZone) => {
  assert.throws(
    RangeError,
    () => Temporal.Duration.compare(new Temporal.Duration(1), new Temporal.Duration(), { relativeTo: { year: 2000, month: 5, day: 2, timeZone } }),
    "reject minus zero as extended year"
  );
});
