// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tostring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const date1 = new Temporal.PlainDate(2000, 5, 2);
const date2 = new Temporal.PlainDate(2000, 5, 2, "gregory");

[
  [date1, "2000-05-02"],
  [date2, "2000-05-02[u-ca=gregory]"],
].forEach(([date, expected]) => {
  const explicit = date.toString(undefined);
  assert.sameValue(explicit, expected, "default calendarName option is auto");

  const implicit = date.toString();
  assert.sameValue(implicit, expected, "default calendarName option is auto");

  const lambda = date.toString(() => {});
  assert.sameValue(lambda, expected, "default calendarName option is auto");
});
