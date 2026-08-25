// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype-@@iterator
description: >
  The method should return a valid iterator with the context as the
  IteratedObject.
features:
  - Symbol.iterator
---*/

var set = new Set();
set.add(1);
set.add(2);
set.add(3);

var iterator = set[Symbol.iterator]();
var result;

result = iterator.next();
assert.sameValue(result.value, 1, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iterator.next();
assert.sameValue(result.value, 2, 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iterator.next();
assert.sameValue(result.value, 3, 'Third result `value`');
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
