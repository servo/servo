// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-performpromiseallsettled
description: >
  Cannot tamper with remainingElementsCount when Promise.allSettled reject element function is called multiple times.
info: |
  Runtime Semantics: PerformPromiseAllSettled ( iteratorRecord, constructor, resultCapability, )

  If alreadyCalled.[[Value]] is true, return undefined.

features: [Promise.allSettled]
---*/

let rejectCallCount = 0;
let returnValue = {};
let error = new Test262Error();

function Constructor(executor) {
  function reject(value) {
    assert.sameValue(value, error);
    rejectCallCount += 1;
    return returnValue;
  }
  executor(() => {throw error}, reject);
}
Constructor.resolve = function(v) {
  return v;
};
Constructor.reject = function(v) {
  return v;
};

let pOnRejected;

let p = {
  then(onResolved, onRejected) {
    pOnRejected = onRejected;
    onResolved();
  }
};

assert.sameValue(rejectCallCount, 0, 'rejectCallCount before call to allSettled()');

Promise.allSettled.call(Constructor, [p]);

assert.sameValue(rejectCallCount, 1, 'rejectCallCount after call to allSettled()');
assert.sameValue(pOnRejected(), undefined);
assert.sameValue(rejectCallCount, 1, 'rejectCallCount after call to pOnRejected()');
pOnRejected();
assert.sameValue(rejectCallCount, 1, 'rejectCallCount after call to pOnRejected()');


