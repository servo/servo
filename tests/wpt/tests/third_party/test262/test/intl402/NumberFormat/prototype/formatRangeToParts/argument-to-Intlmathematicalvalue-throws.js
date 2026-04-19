// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.NumberFormat-formatRangeToParts
description: >
  "formatRangeToParts" basic tests when argument cannot be converted using ToIntlMathematicalValue
info: |
  Intl.NumberFormat.prototype.formatRangeToParts( start, end )
  (...)
  4. Let x be ? ToIntlMathematicalValue(start).
  5. Let y be ? ToIntlMathematicalValue(end).
features: [Intl.NumberFormat-v3]
---*/


const nf = new Intl.NumberFormat();

// Throw if arguments cannot be cast using the method ToIntlMathematicalValue
assert.throws(TypeError, () => { nf.formatRangeToParts(Symbol(102), 201) });
assert.throws(TypeError, () => { nf.formatRangeToParts(102,Symbol(201)) });
