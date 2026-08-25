// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: Checking limits of representable PlainDateTime
features: [Temporal]
---*/

const min = new Temporal.PlainDateTime(-271821, 4, 19, 0, 0, 0, 0, 0, 1);
const max = new Temporal.PlainDateTime(275760, 9, 13, 23, 59, 59, 999, 999, 999);

["reject", "constrain"].forEach((overflow) => {
  assert.throws(
    RangeError,
    () => min.subtract({ nanoseconds: 1 }, { overflow }),
    `subtracting 1 nanosecond beyond minimum limit (overflow = ${overflow})`
  );
  assert.throws(
    RangeError,
    () => max.subtract({ nanoseconds: -1 }, { overflow }),
    `subtracting -1 nanosecond beyond maximum limit (overflow = ${overflow})`
  );
});
