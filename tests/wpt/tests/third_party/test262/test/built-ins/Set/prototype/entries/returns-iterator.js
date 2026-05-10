// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-set.prototype.entries
description: >
    Set.prototype.entries ( )

    ...
    2. Return CreateSetIterator(S, "key+value").


    23.2.5.1 CreateSetIterator Abstract Operation

    ...
    7. Return iterator.


---*/

var set = new Set();
set.add(1);
set.add(2);
set.add(3);

var iterator = set.entries();
var result;

result = iterator.next();
assert.sameValue(result.value[0], 1, 'First result `value` ("key")');
assert.sameValue(result.value[1], 1, 'First result `value` ("value")');
assert.sameValue(result.value.length, 2, 'First result `value` (length)');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iterator.next();
assert.sameValue(result.value[0], 2, 'Second result `value` ("key")');
assert.sameValue(result.value[1], 2, 'Second result `value` ("value")');
assert.sameValue(result.value.length, 2, 'Second result `value` (length)');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iterator.next();
assert.sameValue(result.value[0], 3, 'Third result `value` ("key")');
assert.sameValue(result.value[1], 3, 'Third result `value` ("value")');
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
