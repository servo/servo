// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-generator-function-definitions
description: >
  `yield` expressions may be used as the right-hand-side of other `yield`
  expressions.
flags: [async]
features: [async-iteration]
---*/

var g = async function*() {
  yield yield 1;
};

var iter = g();
iter.next().then(function(result) {
  assert.sameValue(result.value, 1, 'First result `value`');
  assert.sameValue(result.done, false, 'First result `done` flag');
}).then(undefined, $DONE);

iter.next().then(function(result) {
  assert.sameValue(result.value, undefined, 'Second result `value`');
  assert.sameValue(result.done, false, 'Second result `done` flag');
}).then(undefined, $DONE);

iter.next().then(function(result) {
  assert.sameValue(result.value, undefined, 'Third result `value`');
  assert.sameValue(result.done, true, 'Thid result `done` flag');
}).then($DONE, $DONE);
