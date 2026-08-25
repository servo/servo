// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth
description: If calendar name is to be emitted, include additional reference info
features: [Temporal]
---*/

const pym = new Temporal.PlainYearMonth(2019, 10, "iso8601", 31);

assert.sameValue(
  pym.toString({ calendarName: 'always' }),
  "2019-10-31[u-ca=iso8601]",
  "emit year-month-day if calendarName = 'always' (four-argument constructor)"
);

const anotherPYM = Temporal.PlainYearMonth.from("2019-10-31"); // 31 will get dropped

assert.sameValue(
  anotherPYM.toString({ calendarName: 'always' }),
  "2019-10-01[u-ca=iso8601]",
  "emit fallback day if calendarName = 'always' (static from)"
);
