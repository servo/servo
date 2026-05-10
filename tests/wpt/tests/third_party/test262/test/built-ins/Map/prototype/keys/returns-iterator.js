// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-map.prototype.keys
description: >
  Returns an iterator.
info: |
  Map.prototype.keys ( )

  ...
  2. Return CreateMapIterator(M, "key").

  23.1.5.1 CreateMapIterator Abstract Operation

  ...
  7. Return iterator.
---*/

var obj = {};
var map = new Map();
map.set('foo', 1);
map.set(obj, 2);
map.set(map, 3);

var iterator = map.keys();
var result;

result = iterator.next();
assert.sameValue(result.value, 'foo', 'First result `value` ("key")');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iterator.next();
assert.sameValue(result.value, obj, 'Second result `value` ("key")');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iterator.next();
assert.sameValue(result.value, map, 'Third result `value` ("key")');
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
