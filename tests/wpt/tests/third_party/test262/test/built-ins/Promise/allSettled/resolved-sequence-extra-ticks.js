// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettled
description: Resolution ticks are set in a predictable sequence with extra then calls
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

var p1 = new Promise(function(resolve) {
  resolve({});
});

sequence.push(1);

Promise.allSettled([p1]).then(function(resolved) {
  sequence.push(4);
  assert.sameValue(sequence.length, 4);
  checkSequence(sequence, 'Expected Promise.allSettled().then to queue second');
}).catch($DONE);

p1.then(function() {
  sequence.push(3);
  assert.sameValue(sequence.length, 3);
  checkSequence(sequence, 'Expected p1.then to queue first');
}).then(function() {
  sequence.push(5);
  assert.sameValue(sequence.length, 5);
  checkSequence(sequence, 'Expected final then to queue last');
}).then($DONE, $DONE);

sequence.push(2);
