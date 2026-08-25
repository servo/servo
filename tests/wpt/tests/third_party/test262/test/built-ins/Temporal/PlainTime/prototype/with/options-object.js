// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaintime.prototype.with
description: Empty or a function object may be used as options
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const instance = new Temporal.PlainTime();

const result1 = instance.with({ minute: 45 }, {});
TemporalHelpers.assertPlainTime(
  result1, 0, 45, 0, 0, 0, 0,
  "options may be an empty plain object"
);

const result2 = instance.with({ minute: 45 }, () => {});
TemporalHelpers.assertPlainTime(
  result2, 0, 45, 0, 0, 0, 0,
  "options may be a function object"
);
