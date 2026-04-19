// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: Can return subseconds.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const feb20 = Temporal.ZonedDateTime.from("2020-02-01T00:00+01:00[+01:00]");
const feb21 = Temporal.ZonedDateTime.from("2021-02-01T00:00+01:00[+01:00]");

const later = feb20.add({
  days: 1,
  milliseconds: 250,
  microseconds: 250,
  nanoseconds: 250
});

const msDiff = feb20.until(later, { largestUnit: "milliseconds" });
TemporalHelpers.assertDuration(msDiff, 0, 0, 0, 0, 0, 0, 0, 86400250, 250, 250);

const µsDiff = feb20.until(later, { largestUnit: "microseconds" });
TemporalHelpers.assertDuration(µsDiff, 0, 0, 0, 0, 0, 0, 0, 0, 86400250250, 250);

const nsDiff = feb20.until(later, { largestUnit: "nanoseconds" });
TemporalHelpers.assertDuration(nsDiff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 86400250250250);
