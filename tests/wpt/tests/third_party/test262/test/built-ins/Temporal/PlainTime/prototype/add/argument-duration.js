// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.add
description: Duration arguments are supported.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const plainTime = new Temporal.PlainTime(15, 23, 30, 123, 456, 789);
const duration = Temporal.Duration.from("PT16H");
TemporalHelpers.assertPlainTime(plainTime.add(duration),
  7, 23, 30, 123, 456, 789);
