// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.instant.prototype.tostring
description: toString() produces a fractional part of the correct length
features: [Temporal]
---*/

const { Instant } = Temporal;

const isoString = '2020-01-01T23:58:57.012034Z';
const instant = Instant.from(isoString);
const instantIsoStrMicros = instant.toString({
  smallestUnit: 'microseconds'
});

assert.sameValue(instantIsoStrMicros, isoString);
