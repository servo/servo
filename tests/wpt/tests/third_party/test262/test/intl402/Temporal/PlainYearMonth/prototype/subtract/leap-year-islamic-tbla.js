// Copyright (C) 2025 Igalia, S.L., and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.subtract
description: Check various basic calculations involving leap years (islamic-tbla calendar)
features: [Temporal, Intl.Era-monthcode]
includes: [temporalHelpers.js]
---*/

const calendar = "islamic-tbla";
const options = { overflow: "reject" };

// Month 12 (Dhu al-Hijjah) has 29 days in common years and 30 in leap years.
// AH 1442, 1445, and 1447 are leap years.

const years1 = new Temporal.Duration(-1);
const years1n = new Temporal.Duration(1);
const years2 = new Temporal.Duration(-2);
const years3n = new Temporal.Duration(3);

const date144512 = Temporal.PlainYearMonth.from({ year: 1445, monthCode: "M12", calendar }, options);

TemporalHelpers.assertPlainYearMonth(
  date144512.subtract(years1, options),
  1446, 12, "M12", "add 1y in leap year",
  "ah", 1446, null);

TemporalHelpers.assertPlainYearMonth(
  date144512.subtract(years2, options),
  1447, 12, "M12", "add 2y landing in next leap year",
  "ah", 1447, null);

TemporalHelpers.assertPlainYearMonth(
  date144512.subtract(years1n, options),
  1444, 12, "M12", "subtract 1y in leap year",
  "ah", 1444, null);

TemporalHelpers.assertPlainYearMonth(
  date144512.subtract(years3n, options),
  1442, 12, "M12", "subtract 3y landing in previous leap year",
  "ah", 1442, null);
