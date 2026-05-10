// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: with should work for Dangi calendar leap dates
features: [Temporal]
---*/

const calendar = "dangi";

const daysInMonthCases = [
  {
    year: 2001,
    leap: "M04L",
    days: [
      30,
      30,
      30,
      29,
      29,
      30,
      29,
      29,
      30,
      29,
      30,
      29,
      30
    ]
  },
];
for (var {year, leap, days} of daysInMonthCases) {
  const date = Temporal.PlainDateTime.from({
    year,
    month: 1,
    day: 1,
    calendar
  });

  const leapMonth = date.with({ monthCode: leap });
  assert.sameValue(leapMonth.monthCode, leap);

  const {monthsInYear} = date;

  for (var i = monthsInYear, leapMonthIndex = undefined, monthStart = undefined; i >= 1; i--) {
    monthStart = monthStart ? monthStart.add({ months: -1 }) : date.add({ months: monthsInYear - 1 });

    const {month, monthCode, daysInMonth} = monthStart;
    assert.sameValue(month, i);
    assert.sameValue(daysInMonth, days[i - 1]);

    if (monthCode.endsWith("L")) {
      assert.sameValue(date.with({ monthCode }).monthCode, leap);
      leapMonthIndex = i;
    } else {
      if (leapMonthIndex !== undefined && i === leapMonthIndex - 1) {
        const inLeapMonth = monthStart.with({ monthCode: `M${ month.toString().padStart(2, "0") }L` });
        assert.sameValue(inLeapMonth.monthCode, `${ monthCode }L`);
      } else {
        assert.throws(RangeError, () => monthStart.with({ monthCode: `M${ month.toString().padStart(2, "0") }L` }, { overflow: "reject" }));

        if (i === 13) {
          assert.throws(RangeError, () => monthStart.with({ monthCode: `M${ month.toString().padStart(2, "0") }L` }));
        }
      }
    }

    const oneDayPastMonthEnd = monthStart.with({ day: daysInMonth + 1 });
    assert.sameValue(oneDayPastMonthEnd.day, daysInMonth);
    assert.throws(RangeError, () => monthStart.with({ day: daysInMonth + 1 }, { overflow: "reject" }));
  }
}
