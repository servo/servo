// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.formatToParts
description: Test for consistent results between Temporal and DateTimeFormat
features: [Temporal, Intl.Era-monthcode]
locale: [en]
---*/

function toFields(dtf, date) {
  const { epochMilliseconds } = date.withCalendar("iso8601").toZonedDateTime("UTC");
  const parts = dtf.formatToParts(epochMilliseconds);

  const yearPart = parts.find(({ type }) => type === "year");
  const monthPart = parts.find(({ type }) => type === "month");
  const dayPart = parts.find(({ type }) => type === "day");

  const year = +yearPart.value;
  const month = +monthPart.value;
  const day = +dayPart.value;

  assert(Number.isInteger(year), `Formatter should return numeric year for ${date}: ${yearPart.value}`);
  assert(Number.isInteger(month), `Formatter should return numeric month for ${date}: ${monthPart.value}`);
  assert(Number.isInteger(day), `Formatter should return numeric day for ${date}: ${dayPart.value}`);

  return { year, month, day };
}

// All supported calendars have at most 31 days per month.
const maximumDaysPerMonth = 31;

const nonLunisolarCalendars = [
  "buddhist",
  "coptic",
  "ethioaa",
  "ethiopic",
  "gregory",
  "indian",
  "islamic-civil",
  "islamic-tbla",
  "islamic-umalqura",
  "japanese",
  "persian",
  "roc",
];

for (let calendar of nonLunisolarCalendars) {
  const dtf = new Intl.DateTimeFormat("en", {
    calendar,
    timeZone: "UTC",
    year: "numeric",
    month: "numeric",
    day: "numeric",
  });

  // Test near past and near future.
  for (let isoYear = 2050; isoYear >= 1950; --isoYear) {
    const {year} = new Temporal.PlainDate(isoYear, 1, 1, calendar);

    for (let month = 1; month <= 12; ++month) {
      const date = Temporal.PlainDate.from({ calendar, year, month, day: maximumDaysPerMonth });
      const fields = toFields(dtf, date);

      const expectedYear = date.eraYear ?? date.year;
      assert.sameValue(fields.year, expectedYear, `date = ${date}, year`);
      assert.sameValue(fields.month, date.month, `date = ${date}, month`);
      assert.sameValue(fields.day, date.day, `date = ${date}, day`);
    }
  }
}
