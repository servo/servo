// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    Yield expressions are valid yield expression operands.
features: [generators]
---*/

function* g() {
  return yield 1;
}
var iter = g();
var result;

result = iter.next();
assert.sameValue(result.value, 1, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next(3);
assert.sameValue(result.value, 3, 'Final result `value`');
assert.sameValue(result.done, true, 'Final result `done` flag');
