// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: >
  Resolution ticks are set in a predictable sequence of mixed fulfilled and rejected promises
info: |
  Runtime Semantics: PerformPromiseAny ( iteratorRecord, constructor, resultCapability )

  Let remainingElementsCount be a new Record { [[Value]]: 1 }.
  ...

  Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] - 1.
  If remainingElementsCount.[[Value]] is 0, then
    Let error be a newly created AggregateError object.
    Perform ! DefinePropertyOrThrow(error, "errors",
      Property Descriptor {
        [[Configurable]]: true,
        [[Enumerable]]: false,
        [[Writable]]: true,
        [[Value]]: errors
      }).
    Return ? Call(promiseCapability.[[Reject]], undefined, « error »).
  ...
flags: [async]
includes: [promiseHelper.js]
features: [Promise.any]
---*/

let sequence = [];

let p1 = new Promise((_, reject) => {
  reject('');
});
let p2 = new Promise(resolve => {
  resolve('');
});
let p3 = new Promise((_, reject) => {
  reject('');
});

sequence.push(1);

p1.catch(() => {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, 'Expected to be called first.');
}).catch($DONE);

Promise.any([p1, p2, p3]).then(() => {
  sequence.push(6);
  assert.sameValue(sequence.length, 6);
  checkSequence(sequence, 'Expected to be called fourth.');
}).then($DONE, $DONE);

p2.then(() => {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, 'Expected to be called second.');
}).catch($DONE);

sequence.push(2);

p3.catch(() => {
  sequence.push(5);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, 'Expected to be called third.');
}).catch($DONE);
