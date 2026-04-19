// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.compare
description: Non-ASCII minus sign is not acceptable
features: [Temporal]
---*/

const invalidStrings = [
  "1976-11-18T15:23:30.12\u221202:00",
  "\u2212009999-11-18T15:23:30.12",
];

const epoch = new Temporal.Instant(0n);
invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(arg, epoch),
    `variant minus sign: ${arg} (first argument)`
  );

  assert.throws(
    RangeError,
    () => Temporal.Instant.compare(epoch, arg),
    `variant minus sign: ${arg} (second argument)`
  );
});
