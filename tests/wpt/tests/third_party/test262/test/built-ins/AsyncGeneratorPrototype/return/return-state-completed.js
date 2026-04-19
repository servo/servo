// Copyright 2018 Valerie Young. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-return
description: return() results in fullfilled promise when called on completed iterator
info: |
  AsyncGenerator.prototype.return ( value )
  1. Let generator be the this value.
  2. Let completion be Completion{[[Type]]: return, [[Value]]: value, [[Target]]: empty}.
  3. Return ! AsyncGeneratorEnqueue(generator, completion).

  AsyncGeneratorEnqueue ( generator, completion )
  ...
  8. If state is not "executing", then
    a. Perform ! AsyncGeneratorResumeNext(generator).
  ...

  AsyncGeneratorResumeNext:
  If completion.[[Type]] is throw, and generator.[[AsyncGeneratorState]] is
  "completed"
flags: [async]
features: [async-iteration]
---*/

var g = async function*() {};

var iter = g();
iter.next().then(function(result) {
  assert.sameValue(result.value, undefined);
  assert.sameValue(result.done, true);

  iter.return(42).then(function(result) {
    assert.sameValue(result.value, 42)
    assert.sameValue(result.done, true)
  }).then($DONE, $DONE);

}).catch($DONE);
