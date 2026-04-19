// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.until
description: Calendar ID is canonicalized
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2024, 6, "islamic-civil", 8);

[
  "2024-06-08[u-ca=islamicc]",
  { year: 1445, month: 12, calendar: "islamicc" },
].forEach((arg) => {
  const result = instance.until(arg);  // would throw if calendar was not canonicalized
  assert(result.blank, "calendar ID is canonicalized");
});
