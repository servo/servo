// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tostring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const datetime1 = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 650, 0);
const datetime2 = new Temporal.PlainDateTime(2000, 5, 2, 12, 34, 56, 987, 650, 0, "gregory");

[
  [datetime1, "2000-05-02T12:34:56.98765"],
  [datetime2, "2000-05-02T12:34:56.98765[u-ca=gregory]"],
].forEach(([datetime, expected]) => {
  const explicit = datetime.toString(undefined);
  assert.sameValue(explicit, expected, "default calendarName option is auto, precision is auto, and no rounding");

  const propertyImplicit = datetime.toString({});
  assert.sameValue(propertyImplicit, expected, "default calendarName option is auto, precision is auto, and no rounding");

  const implicit = datetime.toString();
  assert.sameValue(implicit, expected, "default calendarName option is auto, precision is auto, and no rounding");
});
