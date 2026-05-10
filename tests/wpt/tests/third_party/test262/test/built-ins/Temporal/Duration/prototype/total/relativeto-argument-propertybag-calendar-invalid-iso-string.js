// Copyright (C) 2025 Brage Hogstad, University of Bergen. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Invalid ISO string as calendar in relativeTo option should throw RangeError
features: [Temporal]
---*/

const instance = new Temporal.Duration(1, 0, 0, 0, 24);

const invalidStrings = [
  ["", "empty string"],
  ["notacal", "Unknown calendar"],
];

for (const [cal, description] of invalidStrings) {
  const arg = { year: 2019, monthCode: "M11", day: 1, calendar: cal };
  assert.throws(
    RangeError,
    () => instance.total({ unit: "months", relativeTo: arg }),
    `${description} is not a valid calendar ID`
  );
}
