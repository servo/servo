// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Tests balancing of days to months at end of month (Hebrew calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "hebrew";
const options = { overflow: "reject" };

// 5784 is a leap year.

// 30-day months: 01, 05, 05L, 07, 09, 11
// 29-day months: 04, 06, 08, 10, 12
//
// Cheshvan and Kislev (02, 03) have 29 or 30 days, independent of leap years.
// Deficient - Cheshvan and Kislev have 29 days
// Regular - Cheshvan has 29 days, Kislev 30
// Complete - Cheshvan and Kislev have 30 days
//
// Some recent years of each type: 
// 5778 - regular common year
// 5779 - complete leap year
// 5781 - deficient common year
// 5782 - regular leap year
// 5783 - complete common year
// 5784 - deficient leap year

// Difference between end of longer month to end of following shorter month
{
  const end = Temporal.PlainDate.from({ year: 5783, monthCode: "M08", day: 29, calendar }, options);
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 5783, monthCode: "M07", day: 29, calendar }, options).until(end, { largestUnit }),
      0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
      `Nisan 29th to Iyar 29th is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 5783, monthCode: "M07", day: 30, calendar }, options).until(end, { largestUnit }),
      0, 0, 0, 29, 0, 0, 0, 0, 0, 0,
      `Nisan 30th to Iyar 29th is 29 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of longer month to end of not-immediately-following
// shorter month
{
  const end = Temporal.PlainDate.from({ year: 5783, monthCode: "M12", day: 29, calendar }, options);
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 5783, monthCode: "M09", day: 29, calendar }, options).until(end, { largestUnit }),
      0, 3, 0, 0, 0, 0, 0, 0, 0, 0,
      `Sivan 29th to Elul 29th is 3 months (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 5783, monthCode: "M09", day: 30, calendar }, options).until(end, { largestUnit }),
      0, 2, 0, 29, 0, 0, 0, 0, 0, 0,
      `Sivan 30th to Elul 29th is 2 months 29 days, not 3 months (${largestUnit})`
    );
  }
}

// Difference between end of longer month in one year to shorter month in
// later year
{
  const end = Temporal.PlainDate.from({ year: 5786, monthCode: "M04", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 5783, monthCode: "M11", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 5783, monthCode: "M11", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "months" }),
    0, 30, 0, 0, 0, 0, 0, 0, 0, 0,
    "Av 29th 5783 to Tevet 29th 5786 is 30 months"
  );
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "years" }),
    2, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    "Av 29th 5783 to Tevet 29th 5786 is 2 years, 5 months"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "months" }),
    0, 29, 0, 29, 0, 0, 0, 0, 0, 0,
    "Av 30th 5783 to Tevet 29th 5786 is 29 months, 29 days, not 30 months"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "years" }),
    2, 4, 0, 29, 0, 0, 0, 0, 0, 0,
    "Av 30th 5783 to Tevet 29th 5786 is 2 years, 4 months, 29 days, not 2 years 5 months"
  );
}

// Difference between 30 Kislev and day 29 of 29-day Kislev in a later year
{
  const end = Temporal.PlainDate.from({ year: 5784, monthCode: "M02", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 5783, monthCode: "M02", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 5783, monthCode: "M02", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "months" }),
    0, 12, 0, 0, 0, 0, 0, 0, 0, 0,
    "29th Kislev 5783 to 29th Kislev 5784 (deficient year) is 12 months"
  );
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "years" }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    "29th Kislev 5783 to 29th Kislev 5784 (deficient year) is 1 year"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "months" }),
    0, 11, 0, 29, 0, 0, 0, 0, 0, 0,
    "30th Kislev 5783 to 29th Kislev 5784 (deficient year) is 11 months 29 days, not 12 months"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "years" }),
    0, 11, 0, 29, 0, 0, 0, 0, 0, 0,
    "30th Kislev 5783 to 29th Kislev 5784 (deficient year) is 11 months 29 days, not 1 year"
  );
}

// Difference between 30 Cheshvan and day 29 of 29-day Cheshvan in a later year
{
  const end = Temporal.PlainDate.from({ year: 5784, monthCode: "M03", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 5783, monthCode: "M03", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 5783, monthCode: "M03", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "months" }),
    0, 12, 0, 0, 0, 0, 0, 0, 0, 0,
    "29th Cheshvan 5783 to 29th Cheshvan 5784 (deficient year) is 12 months"
  );
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "years" }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    "29th Cheshvan 5783 to 29th Cheshvan 5784 (deficient year) is 1 year"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "months" }),
    0, 11, 0, 29, 0, 0, 0, 0, 0, 0,
    "30th Cheshvan 5783 to 29th Cheshvan 5784 (deficient year) is 11 months 29 days, not 12 months"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "years" }),
    0, 11, 0, 29, 0, 0, 0, 0, 0, 0,
    "30th Cheshvan 5783 to 29th Cheshvan 5784 (deficient year) is 11 months 29 days, not 1 year"
  );
}

// Case where both the month and day are not constrained
{
  const end = Temporal.PlainDate.from({ year: 5785, monthCode: "M06", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 5784, monthCode: "M05L", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 5784, monthCode: "M05L", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "months" }),
    0, 13, 0, 0, 0, 0, 0, 0, 0, 0,
    "29th Adar I 5784 to 29th Adar 5785 is 13 months"
  );
  TemporalHelpers.assertDuration(
    start1.until(end, { largestUnit: "years" }),
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    "29th Adar I 5784 to 29th Adar 5785 is 1 year"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "months" }),
    0, 12, 0, 29, 0, 0, 0, 0, 0, 0,
    "30th Adar I 5784 to 29th Adar 5785 is 12 months 29 days, not 13 months"
  );
  TemporalHelpers.assertDuration(
    start2.until(end, { largestUnit: "years" }),
    0, 12, 0, 29, 0, 0, 0, 0, 0, 0,
    "30th Adar I 5784 to 29th Adar 5785 is 12 months 29 days, not 1 year"
  );
}
