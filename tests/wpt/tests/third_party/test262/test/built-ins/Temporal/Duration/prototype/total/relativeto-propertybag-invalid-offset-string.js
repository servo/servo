// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: relativeTo property bag with offset property is rejected if offset is in the wrong format
features: [Temporal]
---*/

const timeZone = "UTC";
const instance = new Temporal.Duration(1, 0, 0, 0, 24);

const badOffsets = [
  "00:00",    // missing sign
  "+0",       // too short
  "-000:00",  // too long
  0,          // must be a string
  null,       // must be a string
  true,       // must be a string
  1000n,      // must be a string
  "+00:0000", // separator must be consistent for hours/minutes and minutes/seconds
];
badOffsets.forEach((offset) => {
  const relativeTo = { year: 2021, month: 10, day: 28, offset, timeZone };
  assert.throws(
    typeof(offset) === 'string' ? RangeError : TypeError,
    () => instance.total({ unit: "days", relativeTo }),
    `"${offset}" is not a valid offset string`
  );
});
