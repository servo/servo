// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.add
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

const date14451230 = Temporal.ZonedDateTime.from({ year: 1445, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }, options);

TemporalHelpers.assertPlainDateTime(
  date14451230.add(years1).toPlainDateTime(),
  1446, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "add 1y to leap day and constrain",
  "ah", 1446);
assert.throws(RangeError, function () {
  date14451230.add(years1, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDateTime(
  date14451230.add(years2, options).toPlainDateTime(),
  1447, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "add 2y to leap day landing in next leap year",
  "ah", 1447);

TemporalHelpers.assertPlainDateTime(
  date14451230.add(years1n).toPlainDateTime(),
  1444, 12, "M12", 29, 12, 34, 0, 0, 0, 0, "subtract 1y from leap day and constrain",
  "ah", 1444);
assert.throws(RangeError, function () {
  date14451230.add(years1n, options);
}, "add 1y to leap day and reject");
TemporalHelpers.assertPlainDateTime(
  date14451230.add(years3n, options).toPlainDateTime(),
  1442, 12, "M12", 30, 12, 34, 0, 0, 0, 0, "subtract 3y from leap day landing in previous leap year",
  "ah", 1442);
