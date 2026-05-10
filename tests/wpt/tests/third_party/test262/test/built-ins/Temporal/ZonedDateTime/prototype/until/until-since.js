// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.until
description: zdt.until(later) === later.since(zdt) with default options.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const zdt = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");

const later = Temporal.ZonedDateTime.from({
  year: 2016,
  month: 3,
  day: 3,
  hour: 18,
  timeZone: "+01:00"
});

TemporalHelpers.assertDurationsEqual(zdt.until(later),
                                     later.since(zdt));
