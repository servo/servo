// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Weeks and months are mutually exclusive.
features: [Temporal]
---*/

// var zdt = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");
const zdt = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");
const laterDateTime = zdt.add({ days: 42, hours: 3});

const weeksDifference = laterDateTime.since(zdt, { largestUnit: "weeks" });
assert.notSameValue(weeksDifference.weeks, 0);
assert.sameValue(weeksDifference.months, 0);

const monthsDifference = laterDateTime.since(zdt, { largestUnit: "months" });
assert.sameValue(monthsDifference.weeks, 0);
assert.notSameValue(monthsDifference.months, 0);
