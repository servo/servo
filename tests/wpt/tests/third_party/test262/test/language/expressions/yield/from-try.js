// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    The behavior of `yield` expressions should not be affected when they appear
    within the `try` block of `try` statements.
features: [generators]
---*/

function* g() {
  try {
    yield 1;
  } catch (err) {
    throw err;
  }
}
var iter = g();
var result;

result = iter.next();
assert.sameValue(result.value, 1, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, undefined, 'Final result `value`');
assert.sameValue(result.done, true, 'Final result `done`flag');
