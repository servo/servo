// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: Non-ASCII minus sign is not acceptable
features: [Temporal]
---*/

const invalidStrings = [
  "1976-11-18T15:23:30.12\u221202:00",
  "1976-11-18T15:23:30.12-02:00[\u221202:00]",
  "1976-11-18T15:23:30.12\u221202:00[\u221202:00]",
  "\u2212009999-11-18T15:23:30.12[UTC]",
];

invalidStrings.forEach((arg) => {
  assert.throws(
    RangeError,
    () => Temporal.ZonedDateTime.from(arg),
    `variant minus sign: ${arg}`
  );
});
