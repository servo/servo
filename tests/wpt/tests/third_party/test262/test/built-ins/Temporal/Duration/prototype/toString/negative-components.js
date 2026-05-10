// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tostring
description: Temporal.Duration.toString handles negative components
features: [Temporal]
---*/
assert.sameValue(
  new Temporal.Duration(-1, -1, -1, -1, -1, -1, -1, -1, -1, -1).toString(),
  "-P1Y1M1W1DT1H1M1.001001001S");
assert.sameValue(
  Temporal.Duration.from({ milliseconds: -250 }).toString(),
  "-PT0.25S");
assert.sameValue(
  Temporal.Duration.from({ milliseconds: -3500 }).toString(),
  "-PT3.5S");
assert.sameValue(
  Temporal.Duration.from({ microseconds: -250 }).toString(),
  "-PT0.00025S");
assert.sameValue(
  Temporal.Duration.from({ microseconds: -3500 }).toString(),
  "-PT0.0035S");
assert.sameValue(
  Temporal.Duration.from({ nanoseconds: -250 }).toString(),
  "-PT0.00000025S");
assert.sameValue(
  Temporal.Duration.from({ nanoseconds: -3500 }).toString(),
  "-PT0.0000035S");
assert.sameValue(
  new Temporal.Duration(0, 0, 0, 0, 0, 0, 0, -1111, -1111, -1111).toString(),
  "-PT1.112112111S");
assert.sameValue(
  Temporal.Duration.from({ seconds: -120, milliseconds: -3500 }).toString(),
  "-PT123.5S");
assert.sameValue(
  Temporal.Duration.from({ weeks: -1, days: -1 }).toString(),
  "-P1W1D");
