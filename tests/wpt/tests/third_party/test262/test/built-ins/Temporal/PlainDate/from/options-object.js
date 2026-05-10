// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.from
description: Empty object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1976, month: 11, day: 18 }, {}), 1976, 11, "M11", 18,
  "options may be an empty plain object"
);

TemporalHelpers.assertPlainDate(
  Temporal.PlainDate.from({ year: 1976, month: 11, day: 18 }, () => {}), 1976, 11, "M11", 18,
  "options may be an empty function object"
);
