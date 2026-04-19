// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.selectRange
description: basic tests internal slot initialization and call receiver errors
info: |
  Intl.PluralRules.prototype.selectRange(start, end )
  (...)
  2. Perform ? RequireInternalSlot(pr, [[InitializedPluralRules]])
features: [Intl.NumberFormat-v3]
---*/

const pr = new Intl.PluralRules();

// Perform ? RequireInternalSlot(pr, [[InitializedPluralRules]]).
let sr = pr['selectRange'];

assert.sameValue(typeof sr, 'function');
assert.throws(TypeError, () => { sr(1, 23) });

