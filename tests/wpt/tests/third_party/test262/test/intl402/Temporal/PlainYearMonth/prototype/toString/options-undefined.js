// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tostring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const yearmonth1 = new Temporal.PlainYearMonth(2000, 5);
const yearmonth2 = new Temporal.PlainYearMonth(2000, 5, "gregory", 1);

[
  [yearmonth1, "2000-05"],
  [yearmonth2, "2000-05-01[u-ca=gregory]"],
].forEach(([yearmonth, expected]) => {
  const explicit = yearmonth.toString(undefined);
  assert.sameValue(explicit, expected, "default calendarName option is auto");

  const implicit = yearmonth.toString();
  assert.sameValue(implicit, expected, "default calendarName option is auto");
});
