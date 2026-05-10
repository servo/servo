// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: RangeError thrown if a string with UTC designator is used as a PlainDate
features: [Temporal, arrow-function]
---*/

const invalidStrings = [
  "2019-10-01T09:00:00Z",
  "2019-10-01T09:00:00Z[UTC]",
];
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.PlainDate.from(arg),
    "String with UTC designator should not be valid as a PlainDate"
  );
});
