// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction
description: >
    When invoked via the function invocation pattern with multiple arguments,
    the AsyncGeneratorFunction intrinsic creates a valid generator whose body is the
    last argument evaluated as source code and whose formal parameters are
    defined by the preceding arguments.
features: [async-iteration]
flags: [async]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

var g = AsyncGeneratorFunction('x', 'y', 'yield x + y;');
var iter = g(2, 3);

iter.next().then(function(result) {
  assert.sameValue(result.value, 5, 'First result `value`');
  assert.sameValue(result.done, false, 'First result `done` flag');
}).then(undefined, $DONE)

iter.next().then(function(result) {
  assert.sameValue(result.value, undefined, 'Final result `value`');
  assert.sameValue(result.done, true, 'Final result `done` flag');
}).then($DONE, $DONE)
