// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Negative zero, as an extended year, is rejected
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "-000000-10-31",
  "-000000-10-31T00:45",
  "-000000-10-31T00:45+01:00",
  "-000000-10-31T00:45+00:00[UTC]",
];

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainDate.from(arg),
    "reject minus zero as extended year"
  );
});
