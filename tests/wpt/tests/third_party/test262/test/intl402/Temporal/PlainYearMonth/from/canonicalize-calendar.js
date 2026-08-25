// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.from
description: Calendar ID is canonicalized
features: [Temporal]
---*/

[
  "2024-06-08[u-ca=islamicc]",
  { year: 1445, month: 12, calendar: "islamicc" },
].forEach((arg) => {
  const result = Temporal.PlainYearMonth.from(arg);
  assert.sameValue(result.calendarId, "islamic-civil", "calendar ID is canonicalized");
});
