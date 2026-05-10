// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.from
description: Calendar ID is canonicalized
features: [Temporal]
---*/

[
  "1972-02-11[u-ca=islamicc]",
  { monthCode: "M12", day: 25, calendar: "islamicc" },
].forEach((arg) => {
  const result = Temporal.PlainMonthDay.from(arg);
  assert.sameValue(result.calendarId, "islamic-civil", "calendar ID is canonicalized");
});
