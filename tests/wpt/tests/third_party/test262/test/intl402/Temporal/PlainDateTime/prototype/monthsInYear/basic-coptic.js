// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.monthsinyear
description: Always 13 months in a year in the Coptic calendar
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "coptic";
const options = { overflow: "reject" };

// 1686 = ISO year 1970

for (var year = 1686; year < 1766; year++) {
    const date = Temporal.PlainDateTime.from({
        year,
        month: 1,
        calendar, day: 1, hour: 12, minute: 34
    });

    assert.sameValue(date.monthsInYear, 13);
}
