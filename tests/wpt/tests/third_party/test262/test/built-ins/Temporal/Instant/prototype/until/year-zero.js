// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.until
description: Negative zero, as an extended year, is rejected
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "-000000-03-30T00:45Z",
  "-000000-03-30T01:45+01:00",
  "-000000-03-30T01:45:00+00:00[UTC]",
];
const instance = new Temporal.Instant(0n);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    "reject minus zero as extended year"
  );
});
