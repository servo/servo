// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withcalendar
description: A time string is valid input for Calendar
features: [Temporal]
---*/

const instance = Temporal.PlainDateTime.from({ year: 1976, month: 11, day: 18, hour: 12, minute: 34});

const calendars = [
  "buddhist",
  "chinese",
  "coptic",
  "dangi",
  "ethioaa",
  "ethiopic",
  "gregory",
  "hebrew",
  "indian",
  "islamic-civil",
  "islamic-tbla",
  "islamic-umalqura",
  "japanese",
  "persian",
  "roc",
]

calendars.forEach((cal) => {
  const str = `T11:30[u-ca=${cal}]`;
  const result = instance.withCalendar(str);
  assert.sameValue(result.calendarId, cal, `Calendar created from string "${str}"`);
});
