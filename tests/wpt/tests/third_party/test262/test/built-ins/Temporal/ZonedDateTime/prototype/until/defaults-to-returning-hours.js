// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Defaults to returning hours.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = Temporal.ZonedDateTime.from("2020-02-01T00:00+01:00[+01:00]");
const feb21 = Temporal.ZonedDateTime.from("2021-02-01T00:00+01:00[+01:00]");

TemporalHelpers.assertDuration(
    feb20.until(feb21),
    0, 0, 0, 0, 8784, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "auto" }),
    0, 0, 0, 0, 8784, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(feb21, { largestUnit: "hours" }),
    0, 0, 0, 0, 8784, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(
    feb20.until(Temporal.ZonedDateTime.from("2021-02-01T00:00:00.000000001+01:00[+01:00]")),
    0, 0, 0, 0, 8784, 0, 0, 0, 0, 1);
TemporalHelpers.assertDuration(
    Temporal.ZonedDateTime.from("2020-02-01T00:00:00.000000001+01:00[+01:00]").until(feb21),
    0, 0, 0, 0, 8783, 59, 59, 999, 999, 999);
