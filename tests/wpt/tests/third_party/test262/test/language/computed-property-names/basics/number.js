// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 12.2.5
description: >
    computed property names can be a number
includes: [compareArray.js]
---*/

function ID(x) {
  return x;
}

var object = {
  a: 'A',
  [1]: 'B',
  c: 'C',
  [ID(2)]: 'D',
};
assert.sameValue(object.a, 'A', "The value of `object.a` is `'A'`. Defined in `object` as `a: 'A'`");
assert.sameValue(object[1], 'B', "The value of `object[1]` is `'B'`. Defined in `object` as `[1]: 'B'`");
assert.sameValue(object.c, 'C', "The value of `object.c` is `'C'`. Defined in `object` as `c: 'C'`");
assert.sameValue(object[2], 'D', "The value of `object[2]` is `'D'`. Defined in `object` as `[ID(2)]: 'D'`");
assert.compareArray(
  Object.getOwnPropertyNames(object),
  ['1', '2', 'a', 'c']
);
