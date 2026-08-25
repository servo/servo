// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.selectRange
description: >
  "selectRange" Throws a RangeError if some of arguments is cast to NaN
info: |
  Intl.PluralRules.prototype.selectRange ( start, end )
  (...)
  WIP: https://github.com/tc39/proposal-intl-numberformat-v3/pull/76


features: [Intl.NumberFormat-v3]
---*/

const pr = new Intl.PluralRules();

assert.throws(RangeError, () => { pr.selectRange(NaN, 100) }, "NaN/Number");
assert.throws(RangeError, () => { pr.selectRange(100, NaN) }, "Number/NaN");
assert.throws(RangeError, () => { pr.selectRange(NaN, NaN) }, "NaN/NaN");
