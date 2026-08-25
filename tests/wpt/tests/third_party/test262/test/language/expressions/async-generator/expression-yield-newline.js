// Copyright 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
author: Caitlin Potter <caitp@igalia.com>
esid: sec-generator-function-definitions
description: >
  Newlines terminate `yield` expressions.
flags: [async]
features: [async-iteration]
---*/

var g = async function*() {
  yield
  1;
};

var iter = g();
iter.next().then(function(result) {
  assert.sameValue(result.value, undefined, 'First result `value`');
  assert.sameValue(result.done, false, 'First result `done` flag');
}).then(undefined, $DONE);

iter.next().then(function(result) {
  assert.sameValue(result.value, undefined, 'Second result `value`');
  assert.sameValue(result.done, true, 'Second result `done` flag');
}).then($DONE, $DONE);
