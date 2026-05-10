// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    When an iterator is the operand of a `yield *` expression, the generator
    should produce an iterator that visits each iterated item.
features: [generators, Symbol.iterator]
---*/

var results = [{ value: 1 }, { value: 8 }, { value: 34, done: true }];
var idx = 0;
var iterator = {};
var iterable = {
  next: function() {
    var result = results[idx];
    idx += 1;
    return result;
  }
};
iterator[Symbol.iterator] = function() {
  return iterable;
};
function* g() {
  yield* iterator;
}
var iter = g();
var result;

result = iter.next();
assert.sameValue(result.value, 1, 'First result `value`');
assert.sameValue(result.done, undefined, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, 8, 'Third result `value`');
assert.sameValue(result.done, undefined, 'Third result `done` flag');

result = iter.next();
assert.sameValue(result.value, undefined, 'Final result `value`');
assert.sameValue(result.done, true, 'Final result `done` flag');
