// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: era and eraYear are ignored (for calendars not using eras)
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const result = Temporal.PlainDateTime.from({
  era: "foobar",
  eraYear: 1,
  year: 1970,
  monthCode: "M01",
  day: 1,
  calendar: "iso8601",
});
TemporalHelpers.assertPlainDateTime(result, 1970, 1, "M01", 1, 0, 0, 0, 0, 0, 0,
  "era and eraYear are ignored for calendar not using eras (iso8601)");

assert.throws(TypeError, () => Temporal.PlainDateTime.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  day: 1,
  calendar: "iso8601",
}), "era and eraYear cannot replace year for calendar not using eras (iso8601)");

const resultChinese = Temporal.PlainDateTime.from({
  era: "foobar",
  eraYear: 1,
  year: 2025,
  monthCode: "M01",
  day: 1,
  calendar: "chinese",
});
TemporalHelpers.assertPlainDateTime(resultChinese, 2025, 1, "M01", 1, 0, 0, 0, 0, 0, 0,
  "era and eraYear are ignored for calendar not using eras (Chinese)");
assert.sameValue(resultChinese.calendarId, "chinese");

assert.throws(TypeError, () => Temporal.PlainDateTime.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  day: 1,
  calendar: "chinese",
}), "era and eraYear cannot replace year for calendar not using eras (Chinese)");
