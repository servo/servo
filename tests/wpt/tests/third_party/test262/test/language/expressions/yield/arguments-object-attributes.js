// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.2
description: >
    Attributes of the `arguments` object are valid yield expression operands.
features: [generators]
---*/

function* g() {
  yield arguments[0];
  yield arguments[1];
  yield arguments[2];
  yield arguments[3];
}
var iter = g(23, 45, 33);
var result;

result = iter.next();
assert.sameValue(result.value, 23, 'First result `value`');
assert.sameValue(result.done, false, 'First result `done` flag');

result = iter.next();
assert.sameValue(result.value, 45, 'Second result `value`');
assert.sameValue(result.done, false, 'Second result `done` flag');

result = iter.next();
assert.sameValue(result.value, 33, 'Third result `value`');
assert.sameValue(result.done, false, 'Third result `done` flag');

result = iter.next();
assert.sameValue(
  result.value, undefined, 'Fourth result `value` (unspecified parameter)'
);
assert.sameValue(
  result.done, false, 'Fourth result `done` flag (unspecified parameter)'
);

result = iter.next();
assert.sameValue(result.value, undefined, 'Final result `value`');
assert.sameValue(result.done, true, 'Final result `done` flag');
