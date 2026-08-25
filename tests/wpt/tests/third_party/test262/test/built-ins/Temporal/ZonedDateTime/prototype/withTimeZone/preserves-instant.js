// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.withtimezone
description: Keeps instant the same.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("2019-11-18T15:23:30.123456789+01:00[+01:00]");
const zdt2 = zdt.withTimeZone("-08:00");

assert.sameValue(zdt.epochNanoseconds, zdt2.epochNanoseconds);
assert.sameValue(zdt2.timeZoneId, "-08:00");

TemporalHelpers.assertPlainDateTime(
    zdt.toPlainDateTime(),
    2019, 11, "M11", 18, 15, 23, 30, 123, 456, 789);
TemporalHelpers.assertPlainDateTime(
    zdt2.toPlainDateTime(),
    2019, 11, "M11", 18, 6, 23, 30, 123, 456, 789);
