// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tostring
description: Empty or a function object may be used as options
features: [Temporal]
---*/

const instance = new Temporal.PlainYearMonth(2019, 10);

const result1 = instance.toString({});
assert.sameValue(
  result1, "2019-10",
  "options may be an empty plain object"
);

const result2 = instance.toString(() => {});
assert.sameValue(
  result2, "2019-10",
  "options may be a function object"
);
