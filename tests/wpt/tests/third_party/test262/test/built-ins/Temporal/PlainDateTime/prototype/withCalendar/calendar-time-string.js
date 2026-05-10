// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.withcalendar
description: A time string is valid input for Calendar; default is iso8601
features: [Temporal]
---*/

const tests = [
  "15:23",
  "15:23:30",
  "15:23:30.123",
  "15:23:30.123456",
  "15:23:30.123456789",
  "1976-11-18T15:23:30.1",
  "1976-11-18T15:23:30.12",
  "1976-11-18T15:23:30.123",
  "1976-11-18T15:23:30.1234",
  "1976-11-18T15:23:30.12345",
  "1976-11-18T15:23:30.123456",
  "1976-11-18T15:23:30.1234567",
  "1976-11-18T15:23:30.12345678",
  "1976-11-18T15:23:30.123456789",
  "1976-11-18T15:23:30,12",
  "1976-11-18T15:23:30.12-02:00",
  "152330",
  "152330.1",
  "152330-08",
  "152330.1-08",
  "152330-0800",
  "152330.1-0800",
  "1976-11-18T152330.1+00:00",
  "19761118T15:23:30.1+00:00",
  "1976-11-18T15:23:30.1+0000",
  "1976-11-18T152330.1+0000",
  "19761118T15:23:30.1+0000",
  "19761118T152330.1+00:00",
  "19761118T152330.1+0000",
  "+001976-11-18T152330.1+00:00",
  "+0019761118T15:23:30.1+00:00",
  "+001976-11-18T15:23:30.1+0000",
  "+001976-11-18T152330.1+0000",
  "+0019761118T15:23:30.1+0000",
  "+0019761118T152330.1+00:00",
  "+0019761118T152330.1+0000",
  "15",
  "T15:23:30",
  "t152330",
];

const instance = Temporal.PlainDateTime.from({ year: 1976, month: 11, day: 18, hour: 12, minute: 34});

tests.forEach((arg) => {
  const result = instance.withCalendar(arg);
  assert.sameValue(result.calendarId, "iso8601", `Calendar created from string "${arg}"`);
});
