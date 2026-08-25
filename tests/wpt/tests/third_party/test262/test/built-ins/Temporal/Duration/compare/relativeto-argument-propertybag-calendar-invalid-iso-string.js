// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: Invalid ISO string as calendar in relativeTo option should throw RangeError
features: [Temporal]
---*/

const instance_1 = new Temporal.Duration(1);
const instance_2 = new Temporal.Duration(2);

const invalidStrings = [
  ["", "empty string"],
  ["notacal", "Unknown calendar"],
];

for (const [cal, description] of invalidStrings) {
  const arg = { year: 2019, monthCode: "M11", day: 1, calendar: cal };
  assert.throws(
    RangeError,
    () => Temporal.Duration.compare(instance_1, instance_2, { relativeTo: arg }),
    `${description} is not a valid calendar ID`
  );
}
