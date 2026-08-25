// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 25.3.1.3
description: >
    Resuming abruptly from a generator in the 'suspendedStart' state should
    honor the abrupt completion and trigger a transition into the 'completed'
    state.
features: [generators]
---*/

var bodyCount = 0;
function* G() {
  bodyCount += 1;
}
var iter = G();
var result;

result = iter.return(56);

assert.sameValue(result.value, 56);
assert.sameValue(result.done, true);
assert.sameValue(
  bodyCount, 0, 'body not evaluated during processing of `return` method'
);

result = iter.next();

assert.sameValue(result.value, undefined, 'Result `value`');
assert.sameValue(result.done, true, 'Result `done` flag');
assert.sameValue(
  bodyCount, 0, 'body not evaluated when "completed" generator is advanced'
);
