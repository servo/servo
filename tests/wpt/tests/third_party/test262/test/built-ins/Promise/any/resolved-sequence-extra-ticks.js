// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: Resolution ticks are set in a predictable sequence with extra then calls
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

let p1 = new Promise(resolve => {
  resolve({});
});

sequence.push(1);

Promise.any([p1]).then((resolved) => {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, 'Expected Promise.any().then to queue second');
}).catch($DONE);

p1.then(() => {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, 'Expected p1.then to queue first');
}).then(() => {
  sequence.push(5);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, 'Expected final then to queue last');
}).then($DONE, $DONE);

sequence.push(2);
