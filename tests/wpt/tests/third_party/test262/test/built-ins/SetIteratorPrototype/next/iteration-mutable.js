// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype-@@iterator
description: >
  When an item is added to the set after the iterator is created but before
  the iterator is "done" (as defined by 23.2.5.2.1), the new item should be
  accessible via iteration. When an item is added to the set after the
  iterator is "done", the new item should not be accessible via iteration.
features: [Symbol.iterator]
---*/

var set = new Set();
set.add(1);
set.add(2);

var iterator = set[Symbol.iterator]();
var result;

result = iterator.next();
assert.sameValue(result.value, 1, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

set.add(3);

result = iterator.next();
assert.sameValue(result.value, 2, 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iterator.next();
assert.sameValue(result.value, 3, 'Third result `value`');
assert.sameValue(result.done, false, 'Third result `done` flag');

result = iterator.next();
assert.sameValue(result.value, undefined, 'Exhausted result `value`');
assert.sameValue(result.done, true, 'Exhausted result `done` flag');

set.add(4);

result = iterator.next();
assert.sameValue(
  result.value, undefined, 'Exhausted result `value` (repeated request)'
);
assert.sameValue(
  result.done, true, 'Exhausted result `done` flag (repeated request)'
);
