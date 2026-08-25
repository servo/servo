// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.since
description: Supports sub-second precision
includes: [temporalHelpers.js]
features: [Temporal, arrow-function]
---*/

const time1 = Temporal.PlainTime.from("10:23:15");
const time2 = Temporal.PlainTime.from("17:15:57.250250250");

TemporalHelpers.assertDuration(time2.since(time1, { largestUnit: "milliseconds" }),
  0, 0, 0, 0, 0, 0, 0, /* milliseconds = */ 24762250, 250, 250, "milliseconds");

TemporalHelpers.assertDuration(time2.since(time1, { largestUnit: "microseconds" }),
  0, 0, 0, 0, 0, 0, 0, /* milliseconds = */ 0, 24762250250, 250, "microseconds");

TemporalHelpers.assertDuration(time2.since(time1, { largestUnit: "nanoseconds" }),
  0, 0, 0, 0, 0, 0, 0, /* milliseconds = */ 0, 0, 24762250250250, "nanoseconds");
