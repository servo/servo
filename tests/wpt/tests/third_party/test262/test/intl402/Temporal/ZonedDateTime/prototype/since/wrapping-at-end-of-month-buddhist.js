// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.since
description: Tests balancing of days to months at end of month (buddhist calendar)
includes: [temporalHelpers.js]
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "buddhist";

// Difference between end of longer month to end of following shorter month
{
  const end = Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M02", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M01", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
      `Jan 28th to Feb 28th is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M01", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, 0, 0, -30, 0, 0, 0, 0, 0, 0,
      `Jan 29th to Feb 28th is 30 days, not one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, 0, 0, -29, 0, 0, 0, 0, 0, 0,
      `Jan 30th to Feb 28th is 29 days, not one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, 0, 0, -28, 0, 0, 0, 0, 0, 0,
      `Jan 31st to Feb 28th is 28 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of leap-year January to end of leap-year February
{
  const end = Temporal.ZonedDateTime.from({ year: 2515, monthCode: "M02", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2515, monthCode: "M01", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, -1, 0, 0, 0, 0, 0, 0, 0, 0,
      `Jan 29th to Feb 29th is one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2515, monthCode: "M01", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, 0, 0, -30, 0, 0, 0, 0, 0, 0,
      `Jan 30th to Feb 29th is 30 days, not one month (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2515, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, 0, 0, -29, 0, 0, 0, 0, 0, 0,
      `Jan 31st to Feb 29th is 29 days, not one month (${largestUnit})`
    );
  }
}

// Difference between end of longer month to end of not-immediately-following
// shorter month
{
  const end = Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M11", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
  for (const largestUnit of ["years", "months"]) {
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M08", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, -3, 0, 0, 0, 0, 0, 0, 0, 0,
      `Aug 30th to Nov 30th is 3 months (${largestUnit})`
    );
    TemporalHelpers.assertDuration(
      Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M08", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit }),
      0, -2, 0, -30, 0, 0, 0, 0, 0, 0,
      `Aug 31st to Nov 30th is 2 months 30 days, not 3 months (${largestUnit})`
    );
  }
}

// Difference between end of longer month in one year to shorter month in
// later year
{
  const end = Temporal.ZonedDateTime.from({ year: 2523, monthCode: "M04", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar });
  TemporalHelpers.assertDuration(
    Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit: "months" }),
    0, -28, 0, 0, 0, 0, 0, 0, 0, 0,
    "Dec 30th 2520 to Apr 30th 2523 is 28 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M12", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit: "years" }),
    -2, -4, 0, 0, 0, 0, 0, 0, 0, 0,
    "Dec 30th 2520 to Apr 30th 2523 is 2 years, 4 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit: "months" }),
    0, -27, 0, -30, 0, 0, 0, 0, 0, 0,
    "Dec 30th 2520 to Apr 30th 2523 is 27 months, 30 days, not 28 months"
  );
  TemporalHelpers.assertDuration(
    Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M12", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar }).since(end, { largestUnit: "years" }),
    -2, -3, 0, -30, 0, 0, 0, 0, 0, 0,
    "Dec 30th 2520 to Apr 30th 2523 is 2 years, 3 months, 30 days, not 2 years 4 months"
  );
}

// Difference where months passes through a month that's the same length or
// shorter than either the start or end month
{
  TemporalHelpers.assertDuration(
    Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M01", day: 29, hour: 12, minute: 34, timeZone: "UTC", calendar })
      .since(Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M03", day: 28, hour: 12, minute: 34, timeZone: "UTC", calendar }), { largestUnit: "months" }),
    0, -1, 0, -28, 0, 0, 0, 0, 0, 0,
    "Jan 29th to Mar 28th is 1 month 28 days, not 58 days"
  );
  TemporalHelpers.assertDuration(
    Temporal.ZonedDateTime.from({ year: 2520, monthCode: "M01", day: 31, hour: 12, minute: 34, timeZone: "UTC", calendar })
      .since(Temporal.ZonedDateTime.from({ year: 2521, monthCode: "M05", day: 30, hour: 12, minute: 34, timeZone: "UTC", calendar }), { largestUnit: "years" }),
    -1, -3, 0, -30, 0, 0, 0, 0, 0, 0,
    "Jan 31st 2520 to May 30th 2521 is 1 year, 3 months, 30 days, not 1 year, 2 months, 60 days"
  );
}
