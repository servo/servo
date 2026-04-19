// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.monthcode
description: iso8601 calendar does not have leap months
features: [Temporal]
---*/


for (var year = 1970; year < 1975; year++) {
  for (var month = 1; month < 13; month++) {
    const date = Temporal.PlainYearMonth.from({
      year,
      month,
      
    });
    assert.sameValue(date.monthCode.endsWith("L"), false);
  }
}
