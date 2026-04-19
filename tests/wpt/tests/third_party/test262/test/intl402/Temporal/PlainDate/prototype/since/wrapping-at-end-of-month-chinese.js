// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.since
description: Tests balancing of days to months at end of month (chinese calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "chinese";
const options = { overflow: "reject" };

// Difference between end of 30-day month to end of following 29-day month
{
  const end = Temporal.PlainDate.from({ year: 2023, monthCode: "M06", day: 29, calendar }, options);
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 2023, monthCode: "M05", day: 29, calendar }, options).since(end, { largestUnit }),
      0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
      `M05-29 (30d) to M06-29 (29d) is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 2023, monthCode: "M05", day: 30, calendar }, options).since(end, { largestUnit }),
      0, 0, 0, -29, 0, 0, 0, 0, 0, 0,
      `M05-30 to M06-29 is 29 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of 30-day M04 to end of 29-day M04L
{
  const end = Temporal.PlainDate.from({ year: 2020, monthCode: "M04L", day: 29, calendar }, options);
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 2020, monthCode: "M04", day: 29, calendar }, options).since(end, { largestUnit }),
      0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
      `M04-29 (30d) to M04L-29 (29d) is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 2020, monthCode: "M04", day: 30, calendar }, options).since(end, { largestUnit }),
      0, 0, 0, -29, 0, 0, 0, 0, 0, 0,
      `M04-30 to M04L-29 (29d) is 29 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of 30-day month to end of not-immediately-following
// 29-day month
{
  const end = Temporal.PlainDate.from({ year: 2023, monthCode: "M09", day: 29, calendar }, options);
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 2023, monthCode: "M04", day: 29, calendar }, options).since(end, { largestUnit }),
      0, -5, 0, 0, 0, 0, 0, 0, 0, 0,
      `M04-29 (30d) to M09-29 (29d) is 5 months (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 2023, monthCode: "M04", day: 30, calendar }, options).since(end, { largestUnit }),
      0, -4, 0, -29, 0, 0, 0, 0, 0, 0,
      `M04-30 to M09-29 (29d) is 4 months 29 days, not 5 months (${largestUnit})`
    );
  }
}

// Difference between end of 30-day month in one year to 29-day month in later
// year
{
  const end = Temporal.PlainDate.from({ year: 2023, monthCode: "M09", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 2021, monthCode: "M05", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 2021, monthCode: "M05", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "months" }),
    0, -29, 0, 0, 0, 0, 0, 0, 0, 0,
    "2021-M05-29 (30d) to 2023-M09-29 (29d) is 29 days"
  );
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "years" }),
    -2, -4, 0, 0, 0, 0, 0, 0, 0, 0,
    "2021-M05-29 (30d) to 2023-M09-29 (29d) is 2 years, 4 months"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "months" }),
    0, -28, 0, -29, 0, 0, 0, 0, 0, 0,
    "2021-M05-30 to 2023-M09-29 (29d) is 28 months, 29 days, not 29 months"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "years" }),
    -2, -3, 0, -29, 0, 0, 0, 0, 0, 0,
    "2021-M05-30 to 2023-M09-29 (29d) is 2 years, 3 months, 29 days, not 2 years 4 months"
  );
}

// Difference between end of 30-day common month and end of the same month with
// 29 days in a later year
{
  const end = Temporal.PlainDate.from({ year: 2019, monthCode: "M02", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 2018, monthCode: "M02", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 2018, monthCode: "M02", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "months" }),
    0, -12, 0, 0, 0, 0, 0, 0, 0, 0,
    "2018-M02-29 to 2019-M02-29 is 12 months"
  );
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "years" }),
    -1, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    "2018-M02-29 to 2019-M02-29 is 1 year"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "months" }),
    0, -11, 0, -29, 0, 0, 0, 0, 0, 0,
    "2018-M02-30 to 2019-M02-29 is 11 months 29 days, not 12 months"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "years" }),
    0, -11, 0, -29, 0, 0, 0, 0, 0, 0,
    "2018-M02-30 to 2019-M02-29 is 11 months 29 days, not 1 year"
  );
}

// Difference between end of 30-day leap month and end of the same leap month
// with 29 days in a later year
{
  const end = Temporal.PlainDate.from({ year: 2025, monthCode: "M06L", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 2017, monthCode: "M06L", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 2017, monthCode: "M06L", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "months" }),
    0, -99, 0, 0, 0, 0, 0, 0, 0, 0,
    "2017-M06L-29 to 2025-M06L-29 is 99 months"
  );
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "years" }),
    -8, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    "2017-M06L-29 to 2025-M06L-29 is 8 years"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "months" }),
    0, -98, 0, -29, 0, 0, 0, 0, 0, 0,
    "2017-M06L-30 to 2025-M06L-29 is 98 months 29 days, not 98 months"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "years" }),
    -7, -12, 0, -29, 0, 0, 0, 0, 0, 0,
    "2017-M06L-30 to 2025-M06L-29 is 7 years 12 months 29 days, not 8 years"
  );
}

// Case where both the month and day are not constrained
{
  const end = Temporal.PlainDate.from({ year: 2018, monthCode: "M06", day: 29, calendar }, options);
  const start1 = Temporal.PlainDate.from({ year: 2017, monthCode: "M06L", day: 29, calendar }, options);
  const start2 = Temporal.PlainDate.from({ year: 2017, monthCode: "M06L", day: 30, calendar }, options);
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "months" }),
    0, -12, 0, 0, 0, 0, 0, 0, 0, 0,
    "2017-M06L-29 to 2018-M06-29 is 12 months"
  );
  TemporalHelpers.assertDuration(
    start1.since(end, { largestUnit: "years" }),
    0, -12, 0, 0, 0, 0, 0, 0, 0, 0,
    "2017-M06L-29 to 2018-M06-29 is 12 months, not 1 year"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "months" }),
    0, -11, 0, -29, 0, 0, 0, 0, 0, 0,
    "2017-M06L-30 to 2018-M06-29 is 11 months 29 days, not 12 months"
  );
  TemporalHelpers.assertDuration(
    start2.since(end, { largestUnit: "years" }),
    0, -11, 0, -29, 0, 0, 0, 0, 0, 0,
    "2017-M06L-30 to 2018-M06-29 is 11 months 29 days, not 1 year"
  );
}
