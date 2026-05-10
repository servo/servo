// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: since() should not return weeks and months together.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const date = new Temporal.PlainDate(1969, 7, 24);
const laterDate = new Temporal.PlainDate(1969, 9, 4);
TemporalHelpers.assertDuration(laterDate.since(date, { largestUnit: "weeks" }),
  0, 0, /* weeks = */ 6, 0, 0, 0, 0, 0, 0, 0, "weeks");
TemporalHelpers.assertDuration(laterDate.since(date, { largestUnit: "months" }),
  0, /* months = */ 1, 0, 11, 0, 0, 0, 0, 0, 0, "months");
