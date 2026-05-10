// Copyright (C) 2022 Chengzhong Wu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncgenerator-prototype-return
description: >
  A broken promise should resume the generator and reject with
  the exception when the generator's state is suspendedYield.
info: |
  AsyncGenerator.prototype.return ( value )
  ...
  9. Else if state is suspendedYield, then
    a. Perform AsyncGeneratorResume(generator, completion).
  ...

  AsyncGeneratorCompleteStep ( generator, completion, done [ , realm ] )
  Resumes the steps defined at AsyncGeneratorStart ( generator, generatorBody )
  ...
  4. Set the code evaluation state of genContext such that when evaluation is resumed for that execution context the following steps will be performed:
    ...
    i. Perform AsyncGeneratorDrainQueue(generator).
    j. Return undefined.

  AsyncGeneratorDrainQueue ( generator )
  ...
  5. Repeat, while done is false,
    a. Let next be the first element of queue.
    b. Let completion be Completion(next.[[Completion]]).
    c. If completion.[[Type]] is return, then
        i. Set generator.[[AsyncGeneratorState]] to awaiting-return.
        ii. Perform AsyncGeneratorAwaitReturn(generator).
        iii. Set done to true.
  ...

flags: [async]
features: [async-iteration]
---*/

let caughtErr;
var g = async function*() {
  try {
    yield;
    return 'this is never returned';
  } catch (err) {
    caughtErr = err;
    return 1;
  }
};

let brokenPromise = Promise.resolve(42);
Object.defineProperty(brokenPromise, 'constructor', {
  get: function () {
    throw new Error('broken promise');
  }
});

var it = g();
it.next().then(() => {
  return it.return(brokenPromise);
}).then(ret => {
  assert.sameValue(caughtErr.message, 'broken promise');
  assert.sameValue(ret.value, 1, 'returned value');
  assert.sameValue(ret.done, true, 'iterator is closed');
}).then($DONE, $DONE);
