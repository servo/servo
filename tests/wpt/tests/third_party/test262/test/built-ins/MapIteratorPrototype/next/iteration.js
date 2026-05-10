// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.1.3.12
description: >
  The method should return a valid iterator with the context as the
  IteratedObject.
features: [Symbol.iterator]
---*/

var map = new Map();
map.set(1, 11);
map.set(2, 22);
map.set(3, 33);

var iterator = map[Symbol.iterator]();
var result;

result = iterator.next();
assert.sameValue(result.value[0], 1, 'First result `value` (map key)');
assert.sameValue(result.value[1], 11, 'First result `value` (map value)');
assert.sameValue(result.value.length, 2, 'First result `value` (length)');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iterator.next();
assert.sameValue(result.value[0], 2, 'Second result `value` (map key)');
assert.sameValue(result.value[1], 22, 'Second result `value` (map value)');
assert.sameValue(result.value.length, 2, 'Second result `value` (length)');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iterator.next();
assert.sameValue(result.value[0], 3, 'Third result `value` (map key)');
assert.sameValue(result.value[1], 33, 'Third result `value` (map value)');
assert.sameValue(result.value.length, 2, 'Third result `value` (length)');
assert.sameValue(result.done, false, 'Third result `done` flag');

result = iterator.next();
assert.sameValue(result.value, undefined, 'Exhausted result `value`');
assert.sameValue(result.done, true, 'Exhausted result `done` flag');

result = iterator.next();
assert.sameValue(
  result.value, undefined, 'Exhausted result `value` (repeated request)'
);
assert.sameValue(
  result.done, true, 'Exhausted result `done` flag (repeated request)'
);
