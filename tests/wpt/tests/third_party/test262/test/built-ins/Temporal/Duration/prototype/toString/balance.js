// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Verify that values are balanced correctly.
features: [Temporal]
---*/

assert.sameValue(
  Temporal.Duration.from({ milliseconds: 3500 }).toString(),
  "PT3.5S");
assert.sameValue(
  Temporal.Duration.from({ microseconds: 3500 }).toString(),
  "PT0.0035S");
assert.sameValue(
  Temporal.Duration.from({ nanoseconds: 3500 }).toString(),
  "PT0.0000035S");
assert.sameValue(
  new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, 1111, 1111, 1111).toString(),
  "PT1.112112111S");
assert.sameValue(
  Temporal.Duration.from({ seconds: 120, milliseconds: 3500 }).toString(),
  "PT123.5S");
