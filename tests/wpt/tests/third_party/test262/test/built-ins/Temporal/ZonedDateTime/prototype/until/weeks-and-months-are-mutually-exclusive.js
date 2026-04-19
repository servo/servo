// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Weeks and months are mutually exclusive.
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");

const laterDateTime = zdt.add({
  days: 42,
  hours: 3
});
const weeksDifference = zdt.until(laterDateTime, { largestUnit: "weeks" });
assert.notSameValue(weeksDifference.weeks, 0);
assert.sameValue(weeksDifference.months, 0);
const monthsDifference = zdt.until(laterDateTime, { largestUnit: "months" });
assert.sameValue(monthsDifference.weeks, 0);
assert.notSameValue(monthsDifference.months, 0);
