// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DateTimeFormat.prototype.formatToParts
description: >
  Test for consistent results between Temporal and DateTimeFormat (lunisolar
  calendars)
features: [Temporal, Intl.Era-monthcode]
locale: [en]
---*/

// Map Hebrew months from English name to their corresponding month code.
const hebrewMonthCodes = {
  Tishri: "M01",
  Heshvan: "M02",
  Kislev: "M03",
  Tevet: "M04",
  Shevat: "M05",
  "Adar I": "M05L",
  Adar: "M06",
  "Adar II": "M06",
  Nisan: "M07",
  Iyar: "M08",
  Sivan: "M09",
  Tamuz: "M10",
  Av: "M11",
  Elul: "M12",
};

function toFieldsLunisolar(dtf, date) {
  const { epochMilliseconds } = date.withCalendar("iso8601").toZonedDateTime("UTC");
  const parts = dtf.formatToParts(epochMilliseconds);

  const yearPart = parts.find(({ type }) => type === "year" || type === "relatedYear");
  const monthPart = parts.find(({ type }) => type === "month");
  const dayPart = parts.find(({ type }) => type === "day");

  const year = +yearPart.value;
  const day = +dayPart.value;
  const month = +monthPart.value;
  let monthCode;
  if (Number.isInteger(month)) {
    monthCode = `M${String(month).padStart(2, "0")}`;
  } else if (date.calendarId === "hebrew") {
    // As per https://unicode-org.atlassian.net/browse/CLDR-15510, the month is
    // never output as numeric in Hebrew.
    monthCode = hebrewMonthCodes[monthPart.value];
  } else {
    const monthNumberPart = Number.parseInt(monthPart.value);
    assert(Number.isInteger(monthNumberPart), `Formatter should return month with numeric part for ${date}: ${monthPart.value}`);
    monthCode = `M${String(monthNumberPart).padStart(2, "0")}L`;
  }

  assert(Number.isInteger(year), `Formatter should return numeric year for ${date}: ${yearPart.value}`);
  assert(Number.isInteger(day), `Formatter should return numeric day for ${date}: ${dayPart.value}`);

  return { year, monthCode, day };
}

// All supported lunisolar calendars have at most 30 days per month.
const maximumDaysPerMonth = 30;

const lunisolarCalendars = [
  "chinese",
  "dangi",
  "hebrew",
];

for (let calendar of lunisolarCalendars) {
  const dtf = new Intl.DateTimeFormat("en", {
    calendar,
    timeZone: "UTC",
    year: "numeric",
    month: "numeric",
    day: "numeric",
  });

  // Test near past and near future.
  for (let isoYear = 2050; isoYear >= 1950; --isoYear) {
    const {year, monthsInYear} = new Temporal.PlainDate(isoYear, 1, 1, calendar);

    for (let month = 1; month <= monthsInYear; ++month) {
      const date = Temporal.PlainDate.from({ calendar, year, month, day: maximumDaysPerMonth });
      const fields = toFieldsLunisolar(dtf, date);

      assert.sameValue(fields.year, date.year, `date = ${date}, year`);
      assert.sameValue(fields.monthCode, date.monthCode, `date = ${date}, monthCode`);
      assert.sameValue(fields.day, date.day, `date = ${date}, day`);
    }
  }
}
