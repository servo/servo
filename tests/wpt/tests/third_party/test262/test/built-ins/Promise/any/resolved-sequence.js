// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.any
description: Resolution ticks are set in a predictable sequence
info: |
  Runtime Semantics: PerformPromiseAny ( iteratorRecord, constructor, resultCapability )

  Let remainingElementsCount be a new Record { [[Value]]: 1 }.
  ...
  6.d ...
    ii. Set remainingElementsCount.[[value]] to remainingElementsCount.[[value]] − 1.
    iii. If remainingElementsCount.[[value]] is 0, then
      Let error be a newly created AggregateError object.
      Perform ! DefinePropertyOrThrow(error, "errors",
        Property Descriptor { [[Configurable]]:  true, [[Enumerable]]: false, [[Writable]]: true, [[Value]]: errors }).
      Return ThrowCompletion(error).
  ...
flags: [async]
includes: [promiseHelper.js]
features: [Promise.any]
---*/

var sequence = [];

var p1 = new Promise(resolve => {
  resolve(1);
});
var p2 = new Promise(resolve => {
  resolve(2);
});

sequence.push(1);

p1.then(function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
checkSequence(sequence, 'Expected to be called first.');
}).catch($DONE);

Promise.any([p1, p2]).then(function() {
  sequence.push(5);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, 'Expected to be called third.');
}).then($DONE, $DONE);

p2.then(function() {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, 'Expected to be called second.');
}).catch($DONE);

sequence.push(2);
