// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.with
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);

const result1 = instance.with({ day: 5 }, {});
TemporalHelpers.assertPlainDate(
  result1, 2000, 5, "M05", 5,
  "options may be an empty plain object"
);

const result2 = instance.with({ day: 5 }, () => {});
TemporalHelpers.assertPlainDate(
  result2, 2000, 5, "M05", 5,
  "options may be a function object"
);
