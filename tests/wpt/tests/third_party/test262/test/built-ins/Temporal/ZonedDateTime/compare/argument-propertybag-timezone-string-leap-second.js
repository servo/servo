// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.compare
description: Leap second is a valid ISO string for TimeZone
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1588377600_000_000_000n, "UTC");

let timeZone = "2016-12-31T23:59:60+00:00[UTC]";

const result1 = Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, instance);
assert.sameValue(result1, 0, "leap second is a valid ISO string for TimeZone (first argument)");
const result2 = Temporal.ZonedDateTime.compare(instance, { year: 2020, month: 5, day: 2, timeZone });
assert.sameValue(result2, 0, "leap second is a valid ISO string for TimeZone (second argument)");

timeZone = "2021-08-19T17:30:45.123456789+23:59[+23:59:60]";
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare({ year: 2020, month: 5, day: 2, timeZone }, instance), "leap second in time zone name not valid (first argument)");
assert.throws(RangeError, () => Temporal.ZonedDateTime.compare(instance, { year: 2020, month: 5, day: 2, timeZone }), "leap second in time zone name not valid (second argument)");
