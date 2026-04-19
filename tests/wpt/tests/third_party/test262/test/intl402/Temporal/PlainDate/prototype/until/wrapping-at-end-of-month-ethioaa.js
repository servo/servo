// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Tests balancing of days to months at end of month (ethioaa calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "ethioaa";

// Difference between end of longer month to end of following shorter month
{
  const end = Temporal.PlainDate.from({ year: 1970, monthCode: "M13", day: 5, calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 5, calendar }).until(end, { largestUnit }),
      0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
      `Mesori 5th to Pikougi Enavot 5th is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 28, calendar }).until(end, { largestUnit }),
      0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
      `Mesori 28th to Pikougi Enavot 5th is 7 days, not one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 29, calendar }).until(end, { largestUnit }),
      0, 0, 0, 6, 0, 0, 0, 0, 0, 0,
      `Mesori 29th to Pikougi Enavot 5th is 6 days, not one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 30, calendar }).until(end, { largestUnit }),
      0, 0, 0, 5, 0, 0, 0, 0, 0, 0,
      `Mesori 30th to Pikougi Enavot 5th is 5 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of leap-year Mesori (M12) to end of leap-year Pikougi Enavot (M13)
{
  const end = Temporal.PlainDate.from({ year: 1971, monthCode: "M13", day: 6, calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1971, monthCode: "M12", day: 6, calendar }).until(end, { largestUnit }),
      0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
      `Mesori 6th to Pikougi Enavot 6th is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1971, monthCode: "M12", day: 29, calendar }).until(end, { largestUnit }),
      0, 0, 0, 7, 0, 0, 0, 0, 0, 0,
      `Mesori 31st to Pikougi Enavot 6th is 7 days, not one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1971, monthCode: "M12", day: 30, calendar }).until(end, { largestUnit }),
      0, 0, 0, 6, 0, 0, 0, 0, 0, 0,
      `Mesori 30th to Pikougi Enavot 6th is 6 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of longer month to end of not-immediately-following
// shorter month
{
  const end = Temporal.PlainDate.from({ year: 1970, monthCode: "M13", day: 5, calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1970, monthCode: "M10", day: 5, calendar }).until(end, { largestUnit }),
      0, 3, 0, 0, 0, 0, 0, 0, 0, 0,
      `Paoni 5th to Pikougi Enavot 5th is 3 months (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.PlainDate.from({ year: 1970, monthCode: "M10", day: 6, calendar }).until(end, { largestUnit }),
      0, 2, 0, 29, 0, 0, 0, 0, 0, 0,
      `Paoni 6th to Pikougi Enavot 5th is 2 months 29 days, not 3 months (${largestUnit})`
    );
  }
}

// Difference between end of longer month in one year to shorter month in
// later year
{
  const end = Temporal.PlainDate.from({ year: 1973, monthCode: "M13", day: 5, calendar });
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 5, calendar }).until(end, { largestUnit: "months" }),
    0, 40, 0, 0, 0, 0, 0, 0, 0, 0,
    "Mesori 5th 1970 to Pikougi Enavot 5th 1973 is 40 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 5, calendar }).until(end, { largestUnit: "years" }),
    3, 1, 0, 0, 0, 0, 0, 0, 0, 0,
    "Mesori 5th 1970 to Pikougi Enavot 5th 1973 is 3 years, 1 month"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 6, calendar }).until(end, { largestUnit: "months" }),
    0, 39, 0, 29, 0, 0, 0, 0, 0, 0,
    "Mesori 6th 1970 to Pikougi Enavot 5th 1973 is 39 months, 29 days, not 40 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1970, monthCode: "M12", day: 7, calendar }).until(end, { largestUnit: "years" }),
    3, 0, 0, 28, 0, 0, 0, 0, 0, 0,
    "Mesori 7th 1970 to Pikougi Enavot 5th 1973 is 3 years, 28 days"
  );
}

// Difference where months passes through a month that's the same length or
// shorter than either the start or end month
{
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1970, monthCode: "M01", day: 29, calendar })
      .until(Temporal.PlainDate.from({ year: 1970, monthCode: "M03", day: 28, calendar }), { largestUnit: "months" }),
    0, 1, 0, 29, 0, 0, 0, 0, 0, 0,
    "Thout 29th to Hathor 28th is 1 month 29 days, not 59 days"
  );
  TemporalHelpers.assertDuration(
    Temporal.PlainDate.from({ year: 1970, monthCode: "M01", day: 30, calendar })
      .until(Temporal.PlainDate.from({ year: 1971, monthCode: "M05", day: 29, calendar }), { largestUnit: "years" }),
    1, 3, 0, 29, 0, 0, 0, 0, 0, 0,
    "Thout 30th 1970 to Tobi 29th 1971 is 1 year, 3 months, 29 days, not 1 year, 2 months, 59 days"
  );
}
