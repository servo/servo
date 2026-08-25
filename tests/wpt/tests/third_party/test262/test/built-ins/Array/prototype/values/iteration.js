// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array.prototype.values
description: >
  The return is a valid iterator with the array's numeric properties.
info: |
  22.1.3.29 Array.prototype.values ( )

  1. Let O be ToObject(this value).
  2. ReturnIfAbrupt(O).
  3. Return CreateArrayIterator(O, "value").
---*/

var array = ['a', 'b', 'c'];
var iterator = array.values();
var result;

result = iterator.next();
assert.sameValue(result.value, 'a', 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iterator.next();
assert.sameValue(result.value, 'b', 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iterator.next();
assert.sameValue(result.value, 'c', 'Third result `value`');
assert.sameValue(result.done, false, 'Third result `done` flag');

result = iterator.next();
assert.sameValue(result.value, undefined, 'Exhausted result `value`');
assert.sameValue(result.done, true, 'Exhausted result `done` flag');
