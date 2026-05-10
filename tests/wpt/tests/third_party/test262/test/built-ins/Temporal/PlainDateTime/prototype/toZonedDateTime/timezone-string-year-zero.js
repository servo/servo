// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Negative zero, as an extended year, is rejected
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "-000000-10-31T17:45Z",
  "-000000-10-31T17:45+00:00[UTC]",
];
const instance = new Temporal.PlainDateTime(2000, 5, 2);
invalidStrings.forEach((timeZone) => {
  assert.throws(
    RangeError,
    () => instance.toZonedDateTime(timeZone),
    "reject minus zero as extended year"
  );
});
