// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction
description: >
    When invoked via the function invocation pattern with a single argument,
    the AsyncGeneratorFunction intrinsic creates a valid async generator whose body is the
    first argument evaluated as source code.
features: [async-iteration]
flags: [async]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

var g = AsyncGeneratorFunction('yield 1;');
var iter = g();
var result;


iter.next().then(function(result) {
  assert.sameValue(result.value, 1, 'First result `value`');
  assert.sameValue(result.done, false, 'First result `done` flag');
}).then(undefined, $DONE)

iter.next().then(function(result) {
  assert.sameValue(result.value, undefined, 'Final result `value`');
  assert.sameValue(result.done, true, 'Final result `done` flag');
}).then($DONE, $DONE)
