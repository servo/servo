// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Overflow option reject works properly.
features: [Temporal]
---*/

const zdt = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789).toZonedDateTime("UTC");

const overflow = "reject";
assert.throws(RangeError, () => zdt.with({ month: 29 }, { overflow }));
assert.throws(RangeError, () => zdt.with({ day: 31 }, { overflow }));
assert.throws(RangeError, () => zdt.with({ hour: 29 }, { overflow }));
assert.throws(RangeError, () => zdt.with({ nanosecond: 9000 }, { overflow }));
