// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.add
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainDate(2000, 5, 2);

const result1 = instance.add({ months: 1 }, {});
TemporalHelpers.assertPlainDate(
  result1, 2000, 6, "M06", 2,
  "options may be an empty plain object"
);

const result2 = instance.add({ months: 1 }, () => {});
TemporalHelpers.assertPlainDate(
  result2, 2000, 6, "M06", 2,
  "options may be a function object"
);
