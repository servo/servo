// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.until
description: Tests balancing of days to months at end of month (Islamic tbla calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "islamic-tbla";

// Difference between end of longer month to end of following shorter month
{
  const end = Temporal.PlainDateTime.from({ year: 1970, monthCode: "M02", day: 29, hour: 12, minute: 34, calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDateTime.from({ year: 1970, monthCode: "M01", day: 29, hour: 12, minute: 34, calendar }).until(end, { largestUnit }),
      0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
      `Muharram 29th to Safar 29th is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDateTime.from({ year: 1970, monthCode: "M01", day: 30, hour: 12, minute: 34, calendar }).until(end, { largestUnit }),
      0, 0, 0, 29, 0, 0, 0, 0, 0, 0,
      `Muharram 30th to Safar 29th is 29 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of longer month to end of not-immediately-following
// shorter month
{
  const end = Temporal.PlainDateTime.from({ year: 1970, monthCode: "M04", day: 29, hour: 12, minute: 34, calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDateTime.from({ year: 1970, monthCode: "M01", day: 29, hour: 12, minute: 34, calendar }).until(end, { largestUnit }),
      0, 3, 0, 0, 0, 0, 0, 0, 0, 0,
      `Muharram 29th to Thani 29th is 3 months (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDateTime.from({ year: 1970, monthCode: "M01", day: 30, hour: 12, minute: 34, calendar }).until(end, { largestUnit }),
      0, 2, 0, 29, 0, 0, 0, 0, 0, 0,
      `Muharram 30th to Thani 29th is 2 months 29 days, not 3 months (${largestUnit})`
    );
  }
}

// Difference between end of longer month in one year to shorter month in
// later year
{
  const end = Temporal.PlainDateTime.from({ year: 1973, monthCode: "M02", day: 29, hour: 12, minute: 34, calendar });
  TemporalHelpers.assertDuration(
    Temporal.PlainDateTime.from({ year: 1970, monthCode: "M11", day: 29, hour: 12, minute: 34, calendar }).until(end, { largestUnit: "months" }),
    0, 27, 0, 0, 0, 0, 0, 0, 0, 0,
    "Qadah 29th 1970 to Safar 29th 1973 is 27 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDateTime.from({ year: 1970, monthCode: "M11", day: 29, hour: 12, minute: 34, calendar }).until(end, { largestUnit: "years" }),
    2, 3, 0, 0, 0, 0, 0, 0, 0, 0,
    "Qadah 29th 1970 to Safar 29th 1973 is 2 years, 3 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDateTime.from({ year: 1970, monthCode: "M11", day: 30, hour: 12, minute: 34, calendar }).until(end, { largestUnit: "months" }),
    0, 26, 0, 29, 0, 0, 0, 0, 0, 0,
    "Qadah 30th 1970 to Safar 29th 1973 is 26 months, 29 days, not 27 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDateTime.from({ year: 1970, monthCode: "M11", day: 30, hour: 12, minute: 34, calendar }).until(end, { largestUnit: "years" }),
    2, 2, 0, 29, 0, 0, 0, 0, 0, 0,
    "Qadah 30th 1970 to Safar 29th 1973 is 2 years, 2 months, 29 days, not 2 years 3 months"
  );
}
