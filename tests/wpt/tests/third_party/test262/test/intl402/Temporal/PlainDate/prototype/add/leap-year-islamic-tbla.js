// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Check various basic calculations involving leap years (islamic-tbla calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "islamic-tbla";
const options = { overflow: "reject" };

// Month 12 (Dhu al-Hijjah) has 29 days in common years and 30 in leap years.
// AH 1442, 1445, and 1447 are leap years.
// See also constrain-day-islamic-tbla.js.

const years1 = new Temporal.Duration(1);
const years1n = new Temporal.Duration(-1);
const years2 = new Temporal.Duration(2);
const years3n = new Temporal.Duration(-3);

const date14451230 = Temporal.PlainDate.from({ year: 1445, monthCode: "M12", day: 30, calendar }, options);

TemporalHelpers.assertPlainDate(
  date14451230.add(years1),
  1446, 12, "M12", 29, "add 1y to leap day and constrain",
  "ah", 1446);
assert.throws(RangeError, function () {
  date14451230.add(years1, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date14451230.add(years2, options),
  1447, 12, "M12", 30, "add 2y to leap day landing in next leap year",
  "ah", 1447);

TemporalHelpers.assertPlainDate(
  date14451230.add(years1n),
  1444, 12, "M12", 29, "subtract 1y from leap day and constrain",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14451230.add(years1n, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDate(
  date14451230.add(years3n, options),
  1442, 12, "M12", 30, "subtract 3y from leap day landing in previous leap year",
  "ah", 1442);
