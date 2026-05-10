// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 26.1.11
description: >
  Returns keys in their corresponding order.
info: |
  26.1.11 Reflect.ownKeys ( target )

  ...
  2. Let keys be target.[[OwnPropertyKeys]]().
  3. ReturnIfAbrupt(keys).
  4. Return CreateArrayFromList(keys).

  9.1.12 [[OwnPropertyKeys]] ( )

  1. Let keys be a new empty List.
  2. For each own property key P of O that is an integer index, in ascending
  numeric index order
    a. Add P as the last element of keys.
  3. For each own property key P of O that is a String but is not an integer
  index, in property creation order
    a. Add P as the last element of keys.
  4. For each own property key P of O that is a Symbol, in property creation
  order
    a. Add P as the last element of keys.
  5. Return keys.
features: [Reflect, Symbol]
---*/


var o = {};
o.p1 = 42;
o.p2 = 43;

var s1 = Symbol('1');
var s2 = Symbol('a');
o[s1] = 44;
o[s2] = 45;

o[2] = 46;
o[0] = 47;
o[1] = 48;

var result = Reflect.ownKeys(o);

assert.sameValue(result.length, 7);
assert.sameValue(result[0], '0');
assert.sameValue(result[1], '1');
assert.sameValue(result[2], '2');
assert.sameValue(result[3], 'p1');
assert.sameValue(result[4], 'p2');
assert.sameValue(result[5], s1);
assert.sameValue(result[6], s2);
