// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.from
description: Trip on ICU4C bug in the epoch year of the Hebrew calendar
info: https://unicode-org.atlassian.net/browse/ICU-23007
features: [Temporal, Intl.Era-monthcode]
---*/

// Rosh Hashanah postponement
{
  // Days in Cheshvan and Kislev for years 0..10.
  const daysPerMonth = {
    Cheshvan: [
      29, 30, 30, 29, 29, 30, 30, 29, 29, 30, 29,
    ],
    Kislev: [
      30, 30, 30, 29, 30, 30, 30, 30, 29, 30, 30,
    ],
  };

  for (let year = 0; year < daysPerMonth.Cheshvan.length; ++year) {
    let endOfCheshvan = Temporal.PlainDate.from({
      calendar: "hebrew",
      year,
      monthCode: "M02",
      day: 30,
    });
    assert.sameValue(endOfCheshvan.day, daysPerMonth.Cheshvan[year]);

    let endOfKislev = Temporal.PlainDate.from({
      calendar: "hebrew",
      year,
      monthCode: "M03",
      day: 30,
    });
    assert.sameValue(endOfKislev.day, daysPerMonth.Kislev[year]);
  }
}

