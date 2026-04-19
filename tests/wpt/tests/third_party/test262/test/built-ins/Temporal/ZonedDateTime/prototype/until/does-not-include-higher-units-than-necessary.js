// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Does not include higher units than necessary in the return value.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const lastFeb20 = Temporal.ZonedDateTime.from("2020-02-29T00:00+01:00[+01:00]");
const lastJan21 = Temporal.ZonedDateTime.from("2021-01-31T00:00+01:00[+01:00]");

TemporalHelpers.assertDuration(lastFeb20.until(lastJan21),
                               0, 0, 0, 0, 8088, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(lastFeb20.until(lastJan21, { largestUnit: "months" }),
                               0, 11, 0, 2, 0, 0, 0, 0, 0, 0);
TemporalHelpers.assertDuration(lastFeb20.until(lastJan21, { largestUnit: "years" }),
                               0, 11, 0, 2, 0, 0, 0, 0, 0, 0);
