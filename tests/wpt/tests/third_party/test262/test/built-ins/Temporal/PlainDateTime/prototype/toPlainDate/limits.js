// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.toplaindate
description: toPlainDate works at the edges of the supported range
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const min = Temporal.PlainDateTime.from('-271821-04-19T00:00:00.000000001');
TemporalHelpers.assertPlainDate(min.toPlainDate(),
  -271821, 4, "M04", 19, "min");

const max = Temporal.PlainDateTime.from('+275760-09-13T23:59:59.999999999');
TemporalHelpers.assertPlainDate(max.toPlainDate(),
  275760, 9, "M09", 13, "max");

