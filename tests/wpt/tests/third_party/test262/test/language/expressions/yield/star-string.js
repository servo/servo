// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    When a string is the operand of a `yield *` expression, the generator
    should produce an iterator that visits each character in order.
features: [generators]
---*/

function* g() {
  yield* 'abc';
}
var iter = g();
var result;

result = iter.next();
assert.sameValue(result.value, 'a', 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, 'b', 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iter.next();
assert.sameValue(result.value, 'c', 'Third result `value`');
assert.sameValue(result.done, false, 'Third result `done` flag');

result = iter.next();
assert.sameValue(result.value, undefined, 'Final result `value`');
assert.sameValue(result.done, true, 'Final result `done` flag');
