// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: era and eraYear are ignored (for calendars not using eras)
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const result = Temporal.PlainYearMonth.from({
  era: "foobar",
  eraYear: 1,
  year: 1970,
  monthCode: "M01",
  calendar: "iso8601",
});
TemporalHelpers.assertPlainYearMonth(result, 1970, 1, "M01",
  "era and eraYear are ignored for calendar not using eras (iso8601)");

assert.throws(TypeError, () => Temporal.PlainYearMonth.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  calendar: "iso8601",
}), "era and eraYear cannot replace year for calendar not using eras (iso8601)");

const resultChinese = Temporal.PlainYearMonth.from({
  era: "foobar",
  eraYear: 1,
  year: 2025,
  monthCode: "M01",
  calendar: "chinese",
});
TemporalHelpers.assertPlainYearMonth(resultChinese, 2025, 1, "M01",
  "era and eraYear are ignored for calendar not using eras (Chinese)",
  undefined, undefined, 29);
assert.sameValue(resultChinese.calendarId, "chinese");

assert.throws(TypeError, () => Temporal.PlainYearMonth.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  day: 1,
  calendar: "chinese",
}), "era and eraYear cannot replace year for calendar not using eras (Chinese)");
