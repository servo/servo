// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.monthcode
description: persian calendar does not have leap months
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "persian";

for (var year = 1348; year < 1428; year++) {
  for (var month = 1; month < 13; month++) {
    const date = Temporal.PlainDateTime.from({
      year,
      month,
      calendar,
      day: 1, hour: 12, minute: 34
    });
    assert.sameValue(date.monthCode.endsWith("L"), false);
  }
}
