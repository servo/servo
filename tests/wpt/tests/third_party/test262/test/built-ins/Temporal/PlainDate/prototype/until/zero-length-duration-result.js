// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: The duration from a date to itself is a zero duration (PT0S)
esid: sec-temporal.plaindate.prototype.until
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const date = new Temporal.PlainDate(2001, 6, 3);

['year', 'month', 'week', 'day'].forEach((largestUnit) => {
  const result = date.until(date, { largestUnit });
  TemporalHelpers.assertDuration(result, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, "The duration from a date to itself is zero")
});
