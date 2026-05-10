// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Adding years works correctly across era boundaries in ethiopic calendar
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const duration5 = new Temporal.Duration(5);
const duration5n = new Temporal.Duration(-5);
const calendar = "ethiopic";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDate.from({ era: "aa", eraYear: 5500, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date1.add(new Temporal.Duration(1)),
  1, 1, "M01", 1, "Adding 1 year to last year of Amete Alem era lands in year 1 of incarnation era",
  "am", 1
);

const date2 = Temporal.PlainDate.from({ era: "am", eraYear: 2000, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date2.add(duration5),
  2005, 6, "M06", 15, "Adding 5 years within incarnation era",
  "am", 2005
);

const date3 = Temporal.PlainDate.from({ era: "aa", eraYear: 5450, monthCode: "M07", day: 12, calendar }, options);
TemporalHelpers.assertPlainDate(
  date3.add(duration5),
  -45, 7, "M07", 12, "Adding 5 years within Amete Alem era",
  "aa", 5455
);

TemporalHelpers.assertPlainDate(
  date2.add(duration5n),
  1995, 6, "M06", 15, "Subtracting 5 years within incarnation era",
  "am", 1995
);

const date4 = Temporal.PlainDate.from({ era: "am", eraYear: 5, monthCode: "M01", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date4.add(duration5n),
  0, 1, "M01", 1, "Subtracting 5 years from year 5 lands in last year of Amete Alem era, arithmetic year 0",
  "aa", 5500
);
