// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Tests balancing of days to months at end of month (indian calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "indian";

// Difference between end of longer month to end of following shorter month
{
  const end = Temporal.PlainDate.from({ year: 1945, monthCode: "M07", day: 30, calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 30, calendar }).until(end, { largestUnit }),
      0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
      `Bhadra 30th to Asvina 30th is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 31, calendar }).until(end, { largestUnit }),
      0, 0, 0, 30, 0, 0, 0, 0, 0, 0,
      `Bhadra 31st to Asvina 30th is 30 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of longer month to end of not-immediately-following
// shorter month
{
  const end = Temporal.PlainDate.from({ year: 1945, monthCode: "M08", day: 30, calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 30, calendar }).until(end, { largestUnit }),
      0, 2, 0, 0, 0, 0, 0, 0, 0, 0,
      `Bhadra 30th to Kartika 30th is 2 months (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 31, calendar }).until(end, { largestUnit }),
      0, 1, 0, 30, 0, 0, 0, 0, 0, 0,
      `Bhadra 31st to Kartika 30th is 1 month 30 days, not 2 months (${largestUnit})`
    );
  }
}

// Difference between end of longer month in one year to shorter month in
// later year
{
  const end = Temporal.PlainDate.from({ year: 1950, monthCode: "M07", day: 30, calendar });
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 30, calendar }).until(end, { largestUnit: "months" }),
    0, 61, 0, 0, 0, 0, 0, 0, 0, 0,
    "Bhadra 30th 1945 to Asvina 30th 1950 is 61 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 30, calendar }).until(end, { largestUnit: "years" }),
    5, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    "Bhadra 30th 1945 to Asvina 30th 1950 is 5 years, 1 month"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 31, calendar }).until(end, { largestUnit: "months" }),
    0, 60, 0, 30, 0, 0, 0, 0, 0, 0,
    "Bhadra 31st 1945 to Asvina 30th 1950 is 60 months, 30 days, not 61 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1945, monthCode: "M06", day: 31, calendar }).until(end, { largestUnit: "years" }),
    5, 0, 0, 30, 0, 0, 0, 0, 0, 0,
    "Bhadra 31st 1945 to Asvina 30th 1950 is 5 years, 30 days"
  );
}
