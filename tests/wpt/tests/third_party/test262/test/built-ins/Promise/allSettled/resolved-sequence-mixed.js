// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: >
  Resolution ticks are set in a predictable sequence of mixed fulfilled and rejected promises
info: |
  Runtime Semantics: PerformPromiseAllSettled ( iteratorRecord, constructor, resultCapability )

  4. Let remainingElementsCount be a new Record { [[Value]]: 1 }.
  ...
  6.d ...
    ii. Set remainingElementsCount.[[value]] to remainingElementsCount.[[value]] − 1.
    iii. If remainingElementsCount.[[value]] is 0, then
      1. Let valuesArray be CreateArrayFromList(values).
      2. Perform ? Call(resultCapability.[[Resolve]], undefined, « valuesArray »).
  ...
flags: [async]
includes: [promiseHelper.js]
features: [Promise.allSettled]
---*/

var sequence = [];

var p1 = new Promise(function(_, reject) {
  reject('');
});
var p2 = new Promise(function(resolve) {
  resolve('');
});
var p3 = new Promise(function(_, reject) {
  reject('');
});

sequence.push(1);

p1.catch(function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, 'Expected to be called first.');
});

Promise.allSettled([p1, p2, p3]).then(function() {
  sequence.push(6);
  assert.sameValue(sequence.length, 6);
  checkSequence(sequence, 'Expected to be called fourth.');
}).then($DONE, $DONE);

p2.then(function() {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, 'Expected to be called second.');
});

sequence.push(2);

p3.catch(function() {
  sequence.push(5);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, 'Expected to be called third.');
});
