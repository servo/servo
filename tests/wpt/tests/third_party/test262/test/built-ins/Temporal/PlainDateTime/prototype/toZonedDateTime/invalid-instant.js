// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tozoneddatetime
description: Convert to zoned datetime outside valid range
features: [Temporal]
---*/

const max = new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999);
const min = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);

assert.throws(
  RangeError,
  () => max.toZonedDateTime("UTC"),
  "outside of Instant range (too big)"
);

assert.throws(
  RangeError,
  () => min.toZonedDateTime("UTC"),
  "outside of Instant range (too small)"
);
