// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainmonthday.prototype.tostring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const tests = [
  [[], "05-02"],
  [["gregory"], "1972-05-02[u-ca=gregory]"],
];

for (const [args, expected] of tests) {
  const monthday = new Temporal.PlainMonthDay(5, 2, ...args);
  const explicit = monthday.toString(undefined);
  assert.sameValue(explicit, expected, "default calendarName option is auto");

  const implicit = monthday.toString();
  assert.sameValue(implicit, expected, "default calendarName option is auto");
}
