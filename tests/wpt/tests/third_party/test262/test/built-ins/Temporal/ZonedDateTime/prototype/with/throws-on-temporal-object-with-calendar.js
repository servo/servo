// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.with
description: Throws if given a Temporal object with a calendar.
features: [Temporal]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

assert.throws(TypeError, () => zdt.with(new Temporal.PlainDateTime(1976, 11, 18, 12, 0)));
assert.throws(TypeError, () => zdt.with(new Temporal.PlainDate(1976, 11, 18)));
assert.throws(TypeError, () => zdt.with(new Temporal.PlainTime(12, 0)));
assert.throws(TypeError, () => zdt.with(new Temporal.PlainYearMonth(1976, 11)));
assert.throws(TypeError, () => zdt.with(new Temporal.PlainMonthDay(11, 18)));
