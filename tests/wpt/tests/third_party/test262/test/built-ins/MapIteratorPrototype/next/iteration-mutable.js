// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 23.1.3.12
description: >
  When an item is added to the map after the iterator is created but before
  the iterator is "done" (as defined by 23.1.5.2.1), the new item should be
  accessible via iteration. When an item is added to the map after the
  iterator is "done", the new item should not be accessible via iteration.
features: [Symbol.iterator]
---*/

var map = new Map();
map.set(1, 11);
map.set(2, 22);

var iterator = map[Symbol.iterator]();
var result;

result = iterator.next();
assert.sameValue(result.value[0], 1, 'First result `value` (map key)');
assert.sameValue(result.value[1], 11, 'First result `value` (map value)');
assert.sameValue(result.value.length, 2, 'First result `value` (length)');
assert.sameValue(result.done, false, 'First result `done` flag');

map.set(3, 33);

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

map.set(4, 44);

result = iterator.next();
assert.sameValue(
  result.value, undefined, 'Exhausted result `value` (repeated request)'
);
assert.sameValue(
  result.done, true, 'Exhausted result `done` flag (repeated request)'
);
