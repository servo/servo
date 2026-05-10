// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    to name, accessor side effects object literal
includes: [compareArray.js]
---*/
var counter = 0;
var key1 = {
  toString: function() {
    assert.sameValue(counter++, 0, "The result of `counter++` is `0`");
    return 'b';
  }
};
var key2 = {
  toString: function() {
    assert.sameValue(counter++, 1, "The result of `counter++` is `1`");
    return 'd';
  }
};
var object = {
  a() { return 'A'; },
  [key1]() { return 'B'; },
  c() { return 'C'; },
  [key2]() { return 'D'; },
};
assert.sameValue(counter, 2, "The value of `counter` is `2`");
assert.sameValue(object.a(), 'A', "`object.a()` returns `'A'`. Defined as `a() { return 'A'; }`");
assert.sameValue(object.b(), 'B', "`object.b()` returns `'B'`. Defined as `[key1]() { return 'B'; }`");
assert.sameValue(object.c(), 'C', "`object.c()` returns `'C'`. Defined as `c() { return 'C'; }`");
assert.sameValue(object.d(), 'D', "`object.d()` returns `'D'`. Defined as `[key2]() { return 'D'; }`");
assert.compareArray(
  Object.getOwnPropertyNames(object),
  ['a', 'b', 'c', 'd']
);
