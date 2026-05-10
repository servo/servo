// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.from
description: Calendar ID is canonicalized
features: [Temporal]
---*/

[
  "2024-07-02T12:34[u-ca=islamicc]",
  { year: 1445, month: 12, day: 25, hour: 12, minute: 34, calendar: "islamicc" },
].forEach((arg) => {
  const result = Temporal.PlainDateTime.from(arg);
  assert.sameValue(result.calendarId, "islamic-civil", "calendar ID is canonicalized");
});
