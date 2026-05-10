// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property method names can be a symbol
includes: [compareArray.js]
features: [Symbol]
---*/

function ID(x) {
  return x;
}

var sym1 = Symbol();
var sym2 = Symbol();
var object = {
  a() { return 'A'; },
  [sym1]() { return 'B'; },
  c() { return 'C'; },
  [ID(sym2)]() { return 'D'; },
};
assert.sameValue(object.a(), 'A', "`object.a()` returns `'A'`. Defined as `a() { return 'A'; }`");
assert.sameValue(object[sym1](), 'B', "`object[sym1]()` returns `'B'`. Defined as `[sym1]() { return 'B'; }`");
assert.sameValue(object.c(), 'C', "`object.c()` returns `'C'`. Defined as `c() { return 'C'; }`");
assert.sameValue(object[sym2](), 'D', "`object[sym2]()` returns `'D'`. Defined as `[ID(sym2)]() { return 'D'; }`");
assert.compareArray(
  Object.getOwnPropertyNames(object),
  ['a', 'c']
);
assert.compareArray(
  Object.getOwnPropertySymbols(object),
  [sym1, sym2]
);
