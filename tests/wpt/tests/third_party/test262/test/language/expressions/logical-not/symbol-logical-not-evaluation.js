// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.5.12.1
description: >
    "Logical Not" coercion operation on Symbols
features: [Symbol]
---*/
var sym = Symbol();

assert.sameValue(!sym, false, "`!sym` is `false`");
assert.sameValue(!!sym, true, "`!!sym` is `true`");

if (!sym) {
  throw new Test262Error("ToBoolean(Symbol) always returns `true`");
} else if (sym) {
  assert(true, "`sym` evaluates to `true`");
} else {
  throw new Test262Error("ToBoolean(Symbol) always returns `true`");
}
