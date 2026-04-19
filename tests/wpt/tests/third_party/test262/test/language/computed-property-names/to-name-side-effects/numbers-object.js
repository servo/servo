// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    to name side effects numbers
includes: [compareArray.js]
---*/
var counter = 0;
var key1 = {
  valueOf: function() {
    assert.sameValue(counter++, 0, "The result of `counter++` is `0`");
    return 1;
  },
  toString: null
};
var key2 = {
  valueOf: function() {
    assert.sameValue(counter++, 1, "The result of `counter++` is `1`");
    return 2;
  },
  toString: null
};

var object = {
  a: 'A',
  [key1]: 'B',
  c: 'C',
  [key2]: 'D',
};
assert.sameValue(counter, 2, "The value of `counter` is `2`");
assert.sameValue(object.a, 'A', "The value of `object.a` is `'A'`. Defined as `a: 'A'`");
assert.sameValue(object[1], 'B', "The value of `object[1]` is `'B'`. Defined as `[key1]: 'B'`");
assert.sameValue(object.c, 'C', "The value of `object.c` is `'C'`. Defined as `c: 'C'`");
assert.sameValue(object[2], 'D', "The value of `object[2]` is `'D'`. Defined as `[key2]: 'D'`");
assert.compareArray(
  Object.getOwnPropertyNames(object),
  ['1', '2', 'a', 'c']
);
