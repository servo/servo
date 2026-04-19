// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday
description: If calendar name is to be emitted, include additional reference info
features: [Temporal]
---*/

const pmd = new Temporal.PlainMonthDay(10, 31, "iso8601", 2019);

assert.sameValue(
  pmd.toString({ calendarName: 'always' }),
  "2019-10-31[u-ca=iso8601]",
  "emit year-month-day if calendarName = 'always' (four-argument constructor)"
);

const anotherPMD = Temporal.PlainMonthDay.from("2019-10-31"); // 2019 will get dropped

assert.sameValue(
  anotherPMD.toString({ calendarName: 'always' }),
  "1972-10-31[u-ca=iso8601]",
  "emit fallback year if calendarName = 'always' (static from)"
);
