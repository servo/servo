// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.from
description: era and eraYear are ignored (for calendars not using eras)
features: [BigInt, Temporal]
---*/

const result = Temporal.ZonedDateTime.from({
  era: "foobar",
  eraYear: 1,
  year: 1970,
  monthCode: "M01",
  day: 1,
  timeZone: "UTC",
  calendar: "iso8601",
});
assert.sameValue(result.epochNanoseconds, 0n,
  "era and eraYear are ignored for calendar not using eras (iso8601)");

assert.throws(TypeError, () => Temporal.ZonedDateTime.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  day: 1,
  timeZone: "UTC",
  calendar: "iso8601",
}), "era and eraYear cannot replace year for calendar not using eras (iso8601)");

const resultChinese = Temporal.ZonedDateTime.from({
  era: "foobar",
  eraYear: 1,
  year: 1969,
  monthCode: "M11",
  day: 24,
  timeZone: "UTC",
  calendar: "chinese",
});
assert.sameValue(resultChinese.epochNanoseconds, 0n,
  "era and eraYear are ignored for calendar not using eras (Chinese)");
assert.sameValue(resultChinese.calendarId, "chinese");

assert.throws(TypeError, () => Temporal.ZonedDateTime.from({
  era: "foobar",
  eraYear: 1,
  monthCode: "M01",
  day: 1,
  timeZone: "UTC",
  calendar: "chinese",
}), "era and eraYear cannot replace year for calendar not using eras (Chinese)");
