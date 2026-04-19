// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Verify that undefined options are handled correctly.
features: [Temporal]
---*/

const datetime1 = new Temporal.ZonedDateTime(957270896_987_650_000n, "UTC");
const datetime2 = new Temporal.ZonedDateTime(957270896_987_650_000n, "UTC", "gregory");

[
  [datetime1, "2000-05-02T12:34:56.98765+00:00[UTC]"],
  [datetime2, "2000-05-02T12:34:56.98765+00:00[UTC][u-ca=gregory]"],
].forEach(([datetime, expected]) => {
  const explicit = datetime.toString(undefined);
  assert.sameValue(explicit, expected, "default show options are auto, precision is auto, and no rounding");

  const propertyImplicit = datetime.toString({});
  assert.sameValue(propertyImplicit, expected, "default show options are auto, precision is auto, and no rounding");

  const implicit = datetime.toString();
  assert.sameValue(implicit, expected, "default show options are auto, precision is auto, and no rounding");
});
