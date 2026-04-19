// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: era and eraYear are ignored (for calendars not using eras)
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const result = Temporal.PlainMonthDay.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  day: 1,
  calendar: "iso8601",
});
TemporalHelpers.assertPlainMonthDay(result, "M01", 1,
  "era and eraYear are ignored for calendar not using eras (iso8601)");

const resultChinese = Temporal.PlainMonthDay.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  day: 1,
  calendar: "chinese",
});
TemporalHelpers.assertPlainMonthDay(resultChinese, "M01", 1,
  "era and eraYear are ignored for calendar not using eras (Chinese)");
assert.sameValue(resultChinese.calendarId, "chinese");
