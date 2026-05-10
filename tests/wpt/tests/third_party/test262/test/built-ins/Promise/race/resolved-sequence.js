// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.race
description: Resolution ticks are set in a predictable sequence
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
includes: [compareArray.js,promiseHelper.js]
---*/

let a = new Promise(resolve => resolve('a'));
let b = new Promise(resolve => resolve('b'));
let sequence = [1];
Promise.all([
  a.then(() => {
    sequence.push(3);
    assert.sameValue(sequence.length, 3);
    return checkSequence(sequence, 'Expected to be called first.');
  }),
  Promise.race([a, b]).then(() => {
    sequence.push(5);
    assert.sameValue(sequence.length, 5);
    return checkSequence(sequence, 'Expected to be called third.');
  }),
  b.then(() => {
    sequence.push(4);
    assert.sameValue(sequence.length, 4);
    return checkSequence(sequence, 'Expected to be called second.');
  })
]).then(result => {
  assert.compareArray(result, [true, true, true]);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence)
}).then($DONE, $DONE);
sequence.push(2);
