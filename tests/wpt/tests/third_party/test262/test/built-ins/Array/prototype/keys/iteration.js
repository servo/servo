// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.keys
description: >
  The return is a valid iterator with the array's numeric properties.
info: |
  22.1.3.13 Array.prototype.keys ( )

  1. Let O be ToObject(this value).
  2. ReturnIfAbrupt(O).
  3. Return CreateArrayIterator(O, "key").
---*/

var array = ['a', 'b', 'c'];
var iterator = array.keys();
var result;

result = iterator.next();
assert.sameValue(result.value, 0, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iterator.next();
assert.sameValue(result.value, 1, 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iterator.next();
assert.sameValue(result.value, 2, 'Third result `value`');
assert.sameValue(result.done, false, 'Third result `done` flag');

result = iterator.next();
assert.sameValue(result.value, undefined, 'Exhausted result `value`');
assert.sameValue(result.done, true, 'Exhausted result `done` flag');
