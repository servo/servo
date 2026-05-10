// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Check constraining days due to leap years (hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// Adar I (M05L) has 30 days, and in common years will be constrained to Adar
// (M06) which has 29 days.
// See also leap-months-hebrew.js and constrain-day-hebrew.js.

const calendar = "hebrew";
const options = { overflow: "reject" };

const adarI = Temporal.PlainDateTime.from({ year: 5782, monthCode: "M05L", day: 30, hour: 12, minute: 34, calendar }, options);

TemporalHelpers.assertPlainDateTime(
  adarI.with({ year: 5783 }),
  5783, 6, "M06", 29,  12, 34, 0, 0, 0, 0,"Changing 30 Adar I to common year constrains to 29 Adar",
  "am", 5783);
assert.throws(RangeError, function () {
  adarI.with({ year: 5783 }, options);
}, "Changing 30 Adar I to common year rejects (either because the month or day would be constrained)");
