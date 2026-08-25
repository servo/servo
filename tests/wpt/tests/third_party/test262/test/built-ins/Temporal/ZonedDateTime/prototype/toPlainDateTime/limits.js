// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaindatetime
description: Checking limits of representable PlainDateTime
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const min = new Temporal.ZonedDateTime(-8_640_000_000_000_000_000_000n, "-23:59");
const max = new Temporal.ZonedDateTime(8_640_000_000_000_000_000_000n, "+23:59");

TemporalHelpers.assertPlainDateTime(
  min.toPlainDateTime(),
  -271821, 4, "M04", 19, 0, 1, 0, 0, 0, 0,
  "minimum"
);

TemporalHelpers.assertPlainDateTime(
  max.toPlainDateTime(),
  275760, 9, "M09", 13, 23, 59, 0, 0, 0, 0,
  "maximum"
);
