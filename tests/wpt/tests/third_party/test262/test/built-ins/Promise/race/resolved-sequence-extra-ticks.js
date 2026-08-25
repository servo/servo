// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.race
description: Resolution ticks are set in a predictable sequence with extra then calls
info: |
  PerformPromiseRace

  Repeat,
    Let next be IteratorStep(iteratorRecord).
    If next is an abrupt completion, set iteratorRecord.[[Done]] to true.
    ReturnIfAbrupt(next).
    If next is false, then
      Set iteratorRecord.[[Done]] to true.
      Return resultCapability.[[Promise]].
    Let nextValue be IteratorValue(next).
    If nextValue is an abrupt completion, set iteratorRecord.[[Done]] to true.
    ReturnIfAbrupt(nextValue).
    Let nextPromise be ? Call(promiseResolve, constructor, « nextValue »).
    Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], resultCapability.[[Reject]] »).

flags: [async]
includes: [promiseHelper.js]
---*/

let a = new Promise(resolve => resolve({}));
let sequence = [1];
Promise.all([
  Promise.race([a]).then(resolved => {
    sequence.push(4);
  }),
  a.then(() => {
    sequence.push(3);
  }).then(() => {
    sequence.push(5);
  }),
]).then(() => {
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence);
}).then($DONE, $DONE);
sequence.push(2);
