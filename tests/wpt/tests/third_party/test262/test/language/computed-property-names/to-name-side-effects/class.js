// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    to name, accessor side effects 3
includes: [compareArray.js]
---*/
var counter = 0;
var key1ToString = [];
var key2ToString = [];
var key1 = {
  toString: function() {
    key1ToString.push(counter);
    counter += 1;
    return 'b';
  }
};
var key2 = {
  toString: function() {
    key2ToString.push(counter);
    counter += 1;
    return 'd';
  }
};
class C {
  a() { return 'A'; }
  [key1]() { return 'B'; }
  c() { return 'C'; }
  [key2]() { return 'D'; }
}

assert.compareArray(key1ToString, [0], "order set for key1");
assert.compareArray(key2ToString, [1], "order set for key2");

assert.sameValue(counter, 2, "The value of `counter` is `2`");
assert.sameValue(new C().a(), 'A', "`new C().a()` returns `'A'`. Defined as `a() { return 'A'; }`");
assert.sameValue(new C().b(), 'B', "`new C().b()` returns `'B'`. Defined as `[key1]() { return 'B'; }`");
assert.sameValue(new C().c(), 'C', "`new C().c()` returns `'C'`. Defined as `c() { return 'C'; }`");
assert.sameValue(new C().d(), 'D', "`new C().d()` returns `'D'`. Defined as `[key2]() { return 'D'; }`");
assert.sameValue(Object.keys(C.prototype).length, 0, "No enum keys from C.prototype");
assert(
  compareArray(Object.getOwnPropertyNames(C.prototype), ['constructor', 'a', 'b', 'c', 'd']),
  "`compareArray(Object.getOwnPropertyNames(C.prototype), ['constructor', 'a', 'b', 'c', 'd'])` returns `true`"
);
