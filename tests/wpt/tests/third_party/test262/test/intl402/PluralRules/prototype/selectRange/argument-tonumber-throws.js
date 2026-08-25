// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.selectRange
description: >
  "selectRange" basic tests when argument cannot be converted using ToIntlMathematicalValue
info: |
  Intl.PluralRules.prototype.selectRange ( start, end )
  (...)
  4. Let x be ? ToIntlMathematicalValue(start).
  5. Let y be ? ToIntlMathematicalValue(end).
locale: [en-US]
features: [Intl.NumberFormat-v3]
---*/

const pr = new Intl.PluralRules("en-US");

// Throw if arguments cannot be cast ToIntlMathematicalValue
assert.throws(TypeError, () => { pr.selectRange(Symbol(102), 201) });
assert.throws(TypeError, () => { pr.selectRange(102,Symbol(201)) });
