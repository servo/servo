// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.compare
description: relativeTo property bag with offset property is rejected if offset is in the wrong format
features: [Temporal]
---*/

const d1 = new Temporal.Duration(0, 1, 0, 280);
const d2 = new Temporal.Duration(0, 1, 0, 281);

const badOffsets = [
  "00:00",    // missing sign
  "+0",       // too short
  "-000:00",  // too long
  1000,       // must be a string
  null,       // must be a string
  true,       // must be a string
  1000n,      // must be a string
  "+00:0000", // separator must be consistent for hours/minutes and minutes/seconds
];
badOffsets.forEach((offset) => {
  const relativeTo = { year: 2021, month: 10, day: 28, offset, timeZone: "UTC" };
  assert.throws(
    typeof(offset) === 'string' ? RangeError : TypeError,
    () => Temporal.Duration.compare(d1, d2, { relativeTo }),
    `"${offset} is not a valid offset string`
  );
});
