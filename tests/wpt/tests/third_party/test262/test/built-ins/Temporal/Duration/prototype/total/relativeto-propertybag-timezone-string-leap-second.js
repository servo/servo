// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.total
description: Leap second is a valid ISO string for TimeZone
features: [Temporal]
---*/

const instance = new Temporal.Duration(1);
let timeZone = "2016-12-31T23:59:60+00:00[UTC]";

// A string with a leap second is a valid ISO string, so the following
// operation should not throw

instance.total({ unit: "months", relativeTo: { year: 2000, month: 5, day: 2, timeZone } });

timeZone = "2021-08-19T17:30:45.123456789+23:59[+23:59:60]";
assert.throws(RangeError, () => instance.total({ unit: "months", relativeTo: { year: 2000, month: 5, day: 2, timeZone } }), "leap second in time zone name not valid");
