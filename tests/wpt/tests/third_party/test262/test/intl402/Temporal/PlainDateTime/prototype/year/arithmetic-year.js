// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.year
description: Arithmetic year calculations for all non-ISO8601 calendars
info: |
  4.1.12 CalendarDateArithmeticYear ( _calendar_, _date_ )

  1. Let _r_ be the row in Table 4 which the value of the Calendar column is
     _calendar_.
  2. Let _epochYear_ be the value given in the "Epoch ISO Year" column of _r_.
  3. Let _epochDate_ be the first day of the calendar year starting in ISO year
     epochYear in the calendar represented by _calendar_, according to
     implementation-defined processing.
  4. Let _newYear_ be the first day of the calendar year of _date_ in the
     calendar represented by _calendar_, according to implementation-defined
     processing.
  5. Let _arithmeticYear_ be the number of whole years between _epochDate_ and
     _newYear_ in the calendar represented by _calendar_, according to
     implementation-defined processing.
  6. Return _arithmeticYear_.
features: [Temporal, Intl.Era-monthcode]
---*/

// Note: year 0 is tested in epoch-year.js

const tests = {
  // One era; arithmetic year equals era year
  buddhist: [
    [{ era: "be", eraYear: -1, monthCode: "M06", day: 27 }, -1],
    [{ era: "be", eraYear: 1, monthCode: "M11", day: 13 }, 1],
    // 2483 would be a short year if the calendar was incorrectly non-proleptic
    [{ era: "be", eraYear: 2483, monthCode: "M02", day: 15 }, 2483],
    [{ era: "be", eraYear: 2567, monthCode: "M08", day: 4 }, 2567],
  ],
  // No eras; we just test that we get back the same arithmetic year we gave
  chinese: [
    [{ year: 2025, monthCode: "M09", day: 26 }, 2025],
  ],
  // One era; arithmetic year equals era year
  coptic: [
    [{ era: "am", eraYear: -1, monthCode: "M04", day: 11 }, -1],
    [{ era: "am", eraYear: 1, monthCode: "M01", day: 12 }, 1],
    [{ era: "am", eraYear: 1742, monthCode: "M03", day: 6 }, 1742],
  ],
  // No eras; we just test that we get back the same arithmetic year we gave
  dangi: [
    [{ year: 2025, monthCode: "M09", day: 26 }, 2025],
  ],
  // One era; arithmetic year equals era year
  ethioaa: [
    [{ era: "aa", eraYear: -1, monthCode: "M04", day: 11 }, -1],
    [{ era: "aa", eraYear: 1, monthCode: "M01", day: 12 }, 1],
    [{ era: "aa", eraYear: 7518, monthCode: "M03", day: 6 }, 7518],
  ],
  ethiopic: [
    [{ era: "aa", eraYear: -1, monthCode: "M02", day: 21 }, -5501],
    [{ era: "aa", eraYear: 0, monthCode: "M04", day: 20 }, -5500],
    [{ era: "aa", eraYear: 1, monthCode: "M13", day: 5 }, -5499],
    [{ era: "aa", eraYear: 5499, monthCode: "M11", day: 16 }, -1],
    [{ era: "am", eraYear: 1, monthCode: "M07", day: 24 }, 1],
    [{ era: "am", eraYear: 2018, monthCode: "M03", day: 6 }, 2018],
  ],
  gregory: [
    [{ era: "bce", eraYear: 2, monthCode: "M06", day: 14 }, -1],
    [{ era: "bce", eraYear: 1, monthCode: "M12", day: 3 }, 0],
    [{ era: "ce", eraYear: 1, monthCode: "M07", day: 26 }, 1],
    [{ era: "ce", eraYear: 2025, monthCode: "M11", day: 15 }, 2025],
  ],
  // One era; arithmetic year equals era year
  hebrew: [
    [{ era: "am", eraYear: -1, monthCode: "M06", day: 2 }, -1],  // fails
    [{ era: "am", eraYear: 1, monthCode: "M09", day: 24 }, 1],
    [{ era: "am", eraYear: 5786, monthCode: "M02", day: 24 }, 5786],
  ],
  // One era; arithmetic year equals era year
  indian: [
    [{ era: "shaka", eraYear: -1, monthCode: "M03", day: 31 }, -1],
    [{ era: "shaka", eraYear: 1, monthCode: "M01", day: 6 }, 1],
    [{ era: "shaka", eraYear: 1947, monthCode: "M08", day: 24 }, 1947],
  ],
  "islamic-civil": [
    [{ era: "bh", eraYear: 2, monthCode: "M08", day: 24 }, -1],
    [{ era: "bh", eraYear: 1, monthCode: "M01", day: 6 }, 0],
    [{ era: "ah", eraYear: 1, monthCode: "M01", day: 5 }, 1],
    [{ era: "ah", eraYear: 1447, monthCode: "M05", day: 24 }, 1447],
  ],
  "islamic-tbla": [
    [{ era: "bh", eraYear: 2, monthCode: "M05", day: 19 }, -1],
    [{ era: "bh", eraYear: 1, monthCode: "M10", day: 16 }, 0],
    [{ era: "ah", eraYear: 1, monthCode: "M12", day: 7 }, 1],
    [{ era: "ah", eraYear: 1447, monthCode: "M05", day: 25 }, 1447],
  ],
  "islamic-umalqura": [
    [{ era: "bh", eraYear: 2, monthCode: "M09", day: 27 }, -1],
    [{ era: "bh", eraYear: 1, monthCode: "M07", day: 17 }, 0],
    [{ era: "ah", eraYear: 1, monthCode: "M11", day: 25 }, 1],
    [{ era: "ah", eraYear: 1447, monthCode: "M05", day: 24 }, 1447],
  ],
  japanese: [
    [{ era: "bce", eraYear: 2, monthCode: "M06", day: 14 }, -1],
    [{ era: "bce", eraYear: 1, monthCode: "M12", day: 3 }, 0],
    [{ era: "ce", eraYear: 1, monthCode: "M07", day: 26 }, 1],
    [{ era: "meiji", eraYear: 6, monthCode: "M12", day: 31 }, 1873],
    [{ era: "taisho", eraYear: 1, monthCode: "M12", day: 31 }, 1912],
    [{ era: "showa", eraYear: 1, monthCode: "M12", day: 31 }, 1926],
    [{ era: "heisei", eraYear: 1, monthCode: "M12", day: 31 }, 1989],
    [{ era: "reiwa", eraYear: 1, monthCode: "M12", day: 31 }, 2019],
    [{ era: "reiwa", eraYear: 7, monthCode: "M11", day: 15 }, 2025],
  ],
  // One era; arithmetic year equals era year
  persian: [
    [{ era: "ap", eraYear: -1, monthCode: "M09", day: 25 }, -1],
    [{ era: "ap", eraYear: 1, monthCode: "M08", day: 22 }, 1],
    [{ era: "ap", eraYear: 1404, monthCode: "M08", day: 24 }, 1404],
  ],
  roc: [
    [{ era: "broc", eraYear: 2, monthCode: "M09", day: 25 }, -1],
    [{ era: "broc", eraYear: 1, monthCode: "M05", day: 3 }, 0],
    [{ era: "roc", eraYear: 1, monthCode: "M12", day: 18 }, 1],
    [{ era: "roc", eraYear: 114, monthCode: "M11", day: 15 }, 114],
  ],
};

for (const [calendar, cases] of Object.entries(tests)) {
  for (const [fromArgs, expectedYear] of cases) {
    const date = Temporal.PlainDateTime.from({ ...fromArgs, hour: 12, minute: 34, calendar }, { overflow: "reject" });
    assert.sameValue(date.year, expectedYear, `${date} should have arithmetic year ${expectedYear}`);
  }
}
