// Copyright (C) 2025 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.duration.prototype.tojson
description: Pairs of units with one large and one small
features: [Temporal]
---*/

assert.sameValue(
  new Temporal.Duration(1, 0, 0, 0, 0, 0, 0, 0, 0, 1).toJSON(),
  "P1YT0.000000001S",
  "years with nanoseconds"
);

assert.sameValue(
  new Temporal.Duration(0, 1, 0, 0, 0, 0, 0, 0, 1).toJSON(),
  "P1MT0.000001S",
  "months with microseconds"
);

assert.sameValue(
  new Temporal.Duration(0, 0, 1, 0, 0, 0, 0, 1).toJSON(),
  "P1WT0.001S",
  "weeks with milliseconds"
);

assert.sameValue(
  new Temporal.Duration(0, 0, 0, 1, 0, 0, 1).toJSON(),
  "P1DT1S",
  "days with seconds"
);

assert.sameValue(
  new Temporal.Duration(0, 0, 0, 0, 1, 1).toJSON(),
  "PT1H1M",
  "hours with minutes"
);
