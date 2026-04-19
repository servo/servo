// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.monthcode
description: Non-lunisolar calendars do not have leap months
features: [Temporal, Intl.Era-monthcode]
---*/

const calendars = {
  "buddhist": 2513,
  "coptic": 1686,
  "ethioaa": 7462,
  "ethiopic": 1962,
  "gregory": 1970,
  "indian": 1892,
  "islamic-civil": 1390,
  "islamic-tbla": 1390,
  "islamic-umalqura": 1390,
  "persian": 1348,
  "roc": 60,
};

for (let [calendar, year] of Object.entries(calendars)) {
  for (var month = 1; month < 13; month++) {
    const date = Temporal.PlainDateTime.from({
        year: year,
        month,
        calendar, day: 1, hour: 12, minute: 34
    });
    assert.sameValue(date.monthCode.endsWith("L"), false);
  }
}
