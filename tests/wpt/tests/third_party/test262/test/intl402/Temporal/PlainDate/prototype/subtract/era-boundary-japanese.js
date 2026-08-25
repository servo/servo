// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
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

const date1 = Temporal.PlainDate.from({ era: "heisei", eraYear: 30, monthCode: "M03", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date1.subtract(duration1),
  2019, 3, "M03", 15, "Adding 1 year to Heisei 30 March (before May 1) lands in Heisei 31 March",
  "heisei", 31
);

const date2 = Temporal.PlainDate.from({ era: "heisei", eraYear: 31, monthCode: "M04", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date2.subtract(duration1),
  2020, 4, "M04", 15, "Adding 1 year to Heisei 31 April (before May 1) lands in Reiwa 2 April",
  "reiwa", 2
);

const date3 = Temporal.PlainDate.from({ era: "heisei", eraYear: 30, monthCode: "M06", day: 12, calendar }, options);
TemporalHelpers.assertPlainDate(
  date3.subtract(duration1),
  2019, 6, "M06", 12, "Adding 1 year to Heisei 30 June (after May 1) lands in Reiwa 1 June",
  "reiwa", 1
);

const date4 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 1, monthCode: "M06", day: 10, calendar }, options);
TemporalHelpers.assertPlainDate(
  date4.subtract(duration1),
  2020, 6, "M06", 10, "Adding 1 year to Reiwa 1 June lands in Reiwa 2 June",
  "reiwa", 2
);

const date5 = Temporal.PlainDate.from({ era: "heisei", eraYear: 28, monthCode: "M07", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date5.subtract(new Temporal.Duration(-3)),
  2019, 7, "M07", 1, "Multiple years across era boundary: Adding 3 years to Heisei 28 July lands in Reiwa 1 July",
  "reiwa", 1
);

const date6 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 2, monthCode: "M06", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date6.subtract(duration1n),
  2019, 6, "M06", 15, "Subtracting 1 year from Reiwa 2 June lands in Reiwa 1 June",
  "reiwa", 1
);

const date7 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 2, monthCode: "M03", day: 15, calendar }, options);
TemporalHelpers.assertPlainDate(
  date7.subtract(duration1n),
  2019, 3, "M03", 15, "Subtracting 1 year from Reiwa 2 March lands in Heisei 31 March",
  "heisei", 31
);

const date8 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 1, monthCode: "M07", day: 10, calendar }, options);
TemporalHelpers.assertPlainDate(
  date8.subtract(duration1n),
  2018, 7, "M07", 10, "Subtracting 1 year from Reiwa 1 July lands in Heisei 30 July",
  "heisei", 30
);

const date9 = Temporal.PlainDate.from({ era: "reiwa", eraYear: 4, monthCode: "M02", day: 1, calendar }, options);
TemporalHelpers.assertPlainDate(
  date9.subtract(new Temporal.Duration(5)),
  2017, 2, "M02", 1, "Subtracting 5 years from Reiwa 4 February lands in Heisei 29 February",
  "heisei", 29
);
