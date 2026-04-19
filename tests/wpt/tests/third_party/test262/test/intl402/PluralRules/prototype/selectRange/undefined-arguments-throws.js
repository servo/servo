// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.selectRange
description: >
  "selectRange" basic tests when arguments are undefined throw a TypeError exception.
info: |
  Intl.PluralRules.prototype.selectRange ( start, end )
  (...)
  3. If start is undefined or end is undefined, throw a TypeError exception.
features: [Intl.NumberFormat-v3]
---*/

const pr = new Intl.PluralRules();

// 1. If arguments are undefined throw a TypeError exception.
assert.throws(TypeError, () => { pr.selectRange(undefined, 201) });
assert.throws(TypeError, () => { pr.selectRange(102, undefined) });
assert.throws(TypeError, () => { pr.selectRange(undefined, undefined)});
