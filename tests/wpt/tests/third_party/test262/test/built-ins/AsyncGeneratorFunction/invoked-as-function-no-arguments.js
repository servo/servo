// Copyright (C) 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-asyncgeneratorfunction
description: >
    When invoked via the function invocation pattern without arguments, the
    AsyncGeneratorFunction intrinsic returns a valid generator with an empty body.
features: [async-iteration]
flags: [async]
---*/

var AsyncGeneratorFunction = Object.getPrototypeOf(async function* () {}).constructor;

var g = AsyncGeneratorFunction();
var iter = g();

iter.next().then(function(result) {
  assert.sameValue(result.value, undefined, 'Result `value`');
  assert.sameValue(result.done, true, 'Result `done` flag');
}).then($DONE, $DONE)
