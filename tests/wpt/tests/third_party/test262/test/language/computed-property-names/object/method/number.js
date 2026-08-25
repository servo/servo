// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property method names can be a number
includes: [compareArray.js]
---*/

function ID(x) {
  return x;
}

var object = {
  a() { return 'A'; },
  [1]() { return 'B'; },
  c() { return 'C'; },
  [ID(2)]() { return 'D'; },
};
assert.sameValue(object.a(), 'A', "`object.a()` returns `'A'`. Defined as `a() { return 'A'; }`");
assert.sameValue(object[1](), 'B', "`object[1]()` returns `'B'`. Defined as `[1]() { return 'B'; }`");
assert.sameValue(object.c(), 'C', "`object.c()` returns `'C'`. Defined as `c() { return 'C'; }`");
assert.sameValue(object[2](), 'D', "`object[2]()` returns `'D'`. Defined as `[ID(2)]() { return 'D'; }`");
assert.compareArray(
  Object.getOwnPropertyNames(object),
  ['1', '2', 'a', 'c']
);
