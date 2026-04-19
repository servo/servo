// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Leap second is a valid ISO string for TimeZone
features: [Temporal]
---*/

const expectedTimeZone = "UTC";
const instance = new Temporal.ZonedDateTime(0n, expectedTimeZone);
let timeZone = "2016-12-31T23:59:60+00:00[UTC]";

// This operation should produce expectedTimeZone, so the following operation
// should not throw due to the time zones being different on the receiver and
// the argument.

instance.until({ year: 2020, month: 5, day: 2, timeZone });

timeZone = "2021-08-19T17:30:45.123456789+23:59[+23:59:60]";
assert.throws(RangeError, () => instance.until({ year: 2020, month: 5, day: 2, timeZone }), "leap second in time zone name not valid");
