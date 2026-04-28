// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Rejecting with a non-thenable object value
esid: sec-promise.any
info: |
  PerformPromiseAny

  Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
  Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

  Promise.any Reject Element Functions

  Let alreadyCalled be F.[[AlreadyCalled]].
  If alreadyCalled.[[Value]] is true, return undefined.
  Set alreadyCalled.[[Value]] to true.
  ...
features: [Promise.any]
---*/

let callCount = 0;
let error;

function Constructor(executor) {
  function reject(result) {
    callCount += 1;
    error = result;
  }
  executor(() => {throw new Test262Error()}, reject);
}
Constructor.resolve = function(v) {
  return v;
};

let p1OnRejected, p2OnRejected, p3OnRejected;

let p1 = {
  then(_, onRejected) {
    p1OnRejected = onRejected;
  }
};
let p2 = {
  then(_, onRejected) {
    p2OnRejected = onRejected;
  }
};
let p3 = {
  then(_, onRejected) {
    p3OnRejected = onRejected;
  }
};

assert.sameValue(callCount, 0, 'callCount before call to any()');

Promise.any.call(Constructor, [p1, p2, p3]);

assert.sameValue(callCount, 0, 'callCount after call to any()');

p1OnRejected('p1-rejection');
p1OnRejected('p1-rejection-unexpected-1');
p1OnRejected('p1-rejection-unexpected-2');

assert.sameValue(callCount, 0, 'callCount after resolving p1');

p2OnRejected('p2-rejection');
p3OnRejected('p3-rejection');

assert.sameValue(callCount, 1, 'callCount after resolving all elements');
