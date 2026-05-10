// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.monthsinyear
description: Always 12 months in a year in the Japanese calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "japanese";
const options = { overflow: "reject" };

// 1892 = ISO year 1970

for (var year = 1892; year < 1972; year++) {
    const date = Temporal.ZonedDateTime.from({
        year,
        month: 1,
        calendar, day: 1, hour: 12, minute: 34, timeZone: "UTC"
    });

    assert.sameValue(date.monthsInYear, 12);
}
