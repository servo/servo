// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.NumberFormat-NumberFormat
description: basic tests internal slot initialization and call receiver errors
info: |
  Intl.NumberFormat.prototype.formatRangeToParts ( start, end )
  (...)
  2. Perform ? RequireInternalSlot(nf, [[InitializedNumberFormat]])
features: [Intl.NumberFormat-v3]
---*/

const nf = new Intl.NumberFormat();

// Perform ? RequireInternalSlot(nf, [[InitializedNumberFormat]])
let f = nf['formatRangeToParts'];

assert.sameValue(typeof f, 'function');
assert.throws(TypeError, () => { f(1, 23) });
