// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.with
description: Verify that undefined options are handled correctly.
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const datetime = new Temporal.PlainDateTime(1976, 11, 18, 15, 23, 30, 123, 456, 789);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ day: 40 }, {}),
  1976, 11, "M11", 30, 15, 23, 30, 123, 456, 789,
  "options may be empty object"
);

TemporalHelpers.assertPlainDateTime(
  datetime.with({ day: 40 }, () => {}),
  1976, 11, "M11", 30, 15, 23, 30, 123, 456, 789,
  "read empty options from function object"
);
