// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.until
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);

const result1 = instance.until(new Temporal.PlainDate(1976, 11, 18), {});
TemporalHelpers.assertDuration(
  result1, 0, 0, 0, -8566, 0, 0, 0, 0, 0, 0,
  "options may be an empty plain object"
);

const result2 = instance.until(new Temporal.PlainDate(1976, 11, 18), () => {});
TemporalHelpers.assertDuration(
  result2, 0, 0, 0, -8566, 0, 0, 0, 0, 0, 0,
  "options may be a function object"
);
