// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.dayofyear
description: dayOfYear property in all non-ISO calendars
features: [Temporal, Intl.Era-monthcode]
---*/

const options = { overflow: "reject" };

// Use equivalent of ISO year 1970
const sampleYears = {
  "buddhist": 2513,
  "chinese": 1969,
  "coptic": 1686,
  "dangi": 1969,
  "ethioaa": 7462,
  "ethiopic": 1962,
  "gregory": 1970,
  "hebrew": 5730,
  "indian": 1891,
  "islamic-civil": 1389,
  "islamic-tbla": 1389,
  "islamic-umalqura": 1389,
  "japanese": 1970,
  "persian": 1348,
  "roc": 59
}

const days1 = new Temporal.Duration(0, 0, 0, 1);

for (var [calendar, year] of Object.entries(sampleYears)) {
  var date = Temporal.ZonedDateTime.from({
    year,
    month: 1,
    day: 1,
    calendar, hour: 12, minute: 34, timeZone: "UTC"
  });

  var expectedDay = 1;

  while (date.year == year) {
    assert.sameValue(date.dayOfYear, expectedDay);
    date = date.add(days1);
    expectedDay++;
  }
}
