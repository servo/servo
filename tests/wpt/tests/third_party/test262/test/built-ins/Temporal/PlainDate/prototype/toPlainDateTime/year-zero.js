// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.toplaindatetime
description: Negative zero, as an extended year, is rejected
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "-000000-12-07T03:24:30",
  "-000000-12-07T03:24:30+01:00",
  "-000000-12-07T03:24:30+00:00[UTC]",
];
const instance = new Temporal.PlainDate(2000, 5, 2);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => instance.toPlainDateTime(arg),
    "reject minus zero as extended year"
  );
});
