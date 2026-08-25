// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: >
  Check various basic calculations involving constraining days to the end of the
  epagomenal month (ethioaa calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "ethioaa";
const options = { overflow: "reject" };

// Years - see leap-year-ethioaa.js
// Months

const common1230 = Temporal.PlainDate.from({ year: 7514, monthCode: "M12", day: 30, calendar }, options);
const leap1230 = Temporal.PlainDate.from({ year: 7515, monthCode: "M12", day: 30, calendar }, options);

TemporalHelpers.assertPlainDate(
  common1230.with({ monthCode: "M13" }),
  7514, 13, "M13", 5, "Changing month constrains to day 5 of common-year epagomenal month",
  "aa", 7514);
assert.throws(RangeError, function () {
  common1230.with({ monthCode: "M13" }, options);
}, "Changing month to common-year epagomenal month rejects");

TemporalHelpers.assertPlainDate(
  leap1230.with({ monthCode: "M13" }),
  7515, 13, "M13", 6, "Changing month constrains to day 6 of leap-year epagomenal month",
  "aa", 7515);
assert.throws(RangeError, function () {
  leap1230.with({ monthCode: "M13" }, options);
}, "Changing month to leap-year epagomenal month rejects");
