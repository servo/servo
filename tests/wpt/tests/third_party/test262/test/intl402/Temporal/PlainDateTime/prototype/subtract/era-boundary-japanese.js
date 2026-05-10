// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.subtract
description: >
  Adding years works correctly across era boundaries in calendars with eras
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

// Reiwa era started on May 1, 2019 (Reiwa 1 = 2019)
// Heisei era: 1989-2019 (Heisei 31 ended April 30, 2019)

const duration1 = new Temporal.Duration(-1);
const duration1n = new Temporal.Duration(1);
const calendar = "japanese";
const options = { overflow: "reject" };

const date1 = Temporal.PlainDateTime.from({ era: "heisei", eraYear: 30, monthCode: "M03", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date1.subtract(duration1),
  2019, 3, "M03", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to Heisei 30 March (before May 1) lands in Heisei 31 March",
  "heisei", 31
);

const date2 = Temporal.PlainDateTime.from({ era: "heisei", eraYear: 31, monthCode: "M04", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date2.subtract(duration1),
  2020, 4, "M04", 15, 12, 34, 0, 0, 0, 0, "Adding 1 year to Heisei 31 April (before May 1) lands in Reiwa 2 April",
  "reiwa", 2
);

const date3 = Temporal.PlainDateTime.from({ era: "heisei", eraYear: 30, monthCode: "M06", day: 12, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date3.subtract(duration1),
  2019, 6, "M06", 12, 12, 34, 0, 0, 0, 0, "Adding 1 year to Heisei 30 June (after May 1) lands in Reiwa 1 June",
  "reiwa", 1
);

const date4 = Temporal.PlainDateTime.from({ era: "reiwa", eraYear: 1, monthCode: "M06", day: 10, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date4.subtract(duration1),
  2020, 6, "M06", 10, 12, 34, 0, 0, 0, 0, "Adding 1 year to Reiwa 1 June lands in Reiwa 2 June",
  "reiwa", 2
);

const date5 = Temporal.PlainDateTime.from({ era: "heisei", eraYear: 28, monthCode: "M07", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date5.subtract(new Temporal.Duration(-3)),
  2019, 7, "M07", 1, 12, 34, 0, 0, 0, 0, "Multiple years across era boundary: Adding 3 years to Heisei 28 July lands in Reiwa 1 July",
  "reiwa", 1
);

const date6 = Temporal.PlainDateTime.from({ era: "reiwa", eraYear: 2, monthCode: "M06", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date6.subtract(duration1n),
  2019, 6, "M06", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from Reiwa 2 June lands in Reiwa 1 June",
  "reiwa", 1
);

const date7 = Temporal.PlainDateTime.from({ era: "reiwa", eraYear: 2, monthCode: "M03", day: 15, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date7.subtract(duration1n),
  2019, 3, "M03", 15, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from Reiwa 2 March lands in Heisei 31 March",
  "heisei", 31
);

const date8 = Temporal.PlainDateTime.from({ era: "reiwa", eraYear: 1, monthCode: "M07", day: 10, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date8.subtract(duration1n),
  2018, 7, "M07", 10, 12, 34, 0, 0, 0, 0, "Subtracting 1 year from Reiwa 1 July lands in Heisei 30 July",
  "heisei", 30
);

const date9 = Temporal.PlainDateTime.from({ era: "reiwa", eraYear: 4, monthCode: "M02", day: 1, hour: 12, minute: 34, calendar }, options);
TemporalHelpers.assertPlainDateTime(
  date9.subtract(new Temporal.Duration(5)),
  2017, 2, "M02", 1, 12, 34, 0, 0, 0, 0, "Subtracting 5 years from Reiwa 4 February lands in Heisei 29 February",
  "heisei", 29
);
