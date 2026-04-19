// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Can return lower or higher units.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = Temporal.ZonedDateTime.from("2020-02-01T00:00+01:00[+01:00]");
const feb21 = Temporal.ZonedDateTime.from("2021-02-01T00:00+01:00[+01:00]");

TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "years" }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "months" }),
    0, 12, 0, 0, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "weeks" }),
    0, 0, 52, 2, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "days" }),
    0, 0, 0, 366, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "minutes" }),
    0, 0, 0, 0, 0, 527040, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "seconds" }),
    0, 0, 0, 0, 0, 0, 31622400, 0, 0, 0);
