// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: zdt.since(earlier) == earlier.until(zdt) with default options.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// var zdt = Temporal.ZonedDateTime.from("1976-11-18T15:23:30.123456789+01:00[+01:00]");
const zdt = new Temporal.ZonedDateTime(217175010123456789n, "+01:00");
const earlier = new Temporal.ZonedDateTime(-120898800000000000n, "+01:00");

TemporalHelpers.assertDurationsEqual(zdt.since(earlier), earlier.until(zdt));


