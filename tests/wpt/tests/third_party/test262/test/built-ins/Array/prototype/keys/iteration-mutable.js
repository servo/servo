// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.keys
description: >
  New items in the array are accessible via iteration until iterator is "done".
info: |
  When an item is added to the array after the iterator is created but
  before the iterator is "done" (as defined by 22.1.5.2.1), the new item's
  key should be accessible via iteration. When an item is added to the
  array after the iterator is "done", the new item's key should not be
  accessible via iteration.
---*/

var array = [];
var iterator = array.keys();
var result;

array.push('a');

result = iterator.next();
assert.sameValue(result.done, false, 'First result `done` flag');
assert.sameValue(result.value, 0, 'First result `value`');

result = iterator.next();
assert.sameValue(result.done, true, 'Exhausted result `done` flag');
assert.sameValue(result.value, undefined, 'Exhausted result `value`');

array.push('b');

result = iterator.next();
assert.sameValue(
  result.done, true,
  'Exhausted result `done` flag (after push)'
);
assert.sameValue(
  result.value, undefined,
  'Exhausted result `value` (after push)'
);
