// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Non-ASCII minus sign is not acceptable
features: [Temporal]
---*/

const invalidStrings = [
  "1976-11-18T15:23:30.12\u221202:00",
  "\u2212009999-11-18T15:23:30.12",
];
const instance = new Temporal.PlainDate(2000, 5, 2);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => instance.until(arg),
    `variant minus sign: ${arg}`
  );
});
