// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: Construction and properties.
features: [Temporal]
---*/

const epochMillis = Date.UTC(1976, 10, 18, 15, 23, 30, 123);
const epochNanos = BigInt(epochMillis) * BigInt(1000000) + BigInt(456789);

// works
var zdt = new Temporal.ZonedDateTime(epochNanos, "-08:00");
assert(zdt instanceof Temporal.ZonedDateTime);
assert.sameValue(typeof zdt, "object");
assert.sameValue(zdt.toInstant().epochMilliseconds, Date.UTC(1976, 10, 18, 15, 23, 30, 123), "epochMilliseconds");

// Temporal.ZonedDateTime for (1976, 11, 18, 15, 23, 30, 123, 456, 789)"
zdt = new Temporal.ZonedDateTime(epochNanos, "UTC");
// can be constructed
assert(zdt instanceof Temporal.ZonedDateTime);
assert.sameValue(typeof zdt, "object");

assert.sameValue(zdt.year, 1976)
assert.sameValue(zdt.month, 11);
assert.sameValue(zdt.monthCode, "M11");
assert.sameValue(zdt.day, 18);
assert.sameValue(zdt.hour, 15);
assert.sameValue(zdt.minute, 23);
assert.sameValue(zdt.second, 30);
assert.sameValue(zdt.millisecond, 123);
assert.sameValue(zdt.microsecond, 456);
assert.sameValue(zdt.nanosecond, 789);
assert.sameValue(zdt.epochMilliseconds, 217178610123);
assert.sameValue(zdt.epochNanoseconds, 217178610123456789n);
assert.sameValue(zdt.dayOfWeek, 4);
assert.sameValue(zdt.dayOfYear, 323);
assert.sameValue(zdt.weekOfYear, 47);
assert.sameValue(zdt.daysInWeek, 7);
assert.sameValue(zdt.daysInMonth, 30);
assert.sameValue(zdt.daysInYear, 366);
assert.sameValue(zdt.monthsInYear, 12);
assert.sameValue(zdt.inLeapYear, true);
assert.sameValue(zdt.offset, "+00:00");
assert.sameValue(zdt.offsetNanoseconds, 0);
assert.sameValue(`${ zdt }`, "1976-11-18T15:23:30.123456789+00:00[UTC]");
