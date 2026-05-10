// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Constraining/rejecting with at month boundaries
features: [Temporal, Intl.Era-monthcode]
---*/

const calendar = "gregory";
const options = { overflow: "reject" };
const year = 1970;

for (var month = 1; month < 13; month++) {
  const date = Temporal.PlainDate.from({
    year,
    month,
    day: 1,
    calendar
  });
  const daysInMonth = date.daysInMonth;

  const oneDayPastMonthEnd = date.with({ day: daysInMonth + 1 });
  assert.sameValue(oneDayPastMonthEnd.day, daysInMonth);
  assert.throws(RangeError, () => date.with({ day: daysInMonth + 1 }, options));
}

