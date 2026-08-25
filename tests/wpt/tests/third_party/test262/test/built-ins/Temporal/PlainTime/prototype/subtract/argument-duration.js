// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.subtract
description: Duration arguments are supported.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
const duration = Temporal.Duration.from("PT16H");
TemporalHelpers.assertPlainTime(plainTime.subtract(duration),
  23, 23, 30, 123, 456, 789);
