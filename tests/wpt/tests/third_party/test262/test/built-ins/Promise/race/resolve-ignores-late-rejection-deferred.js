// Copyright (C) 2020 Rick Waldron, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Resolved promises ignore rejections through deferred invocation of the
    provided resolving function
esid: sec-promise.race
info: |
  Let result be PerformPromiseRace(iteratorRecord, C, promiseCapability, promiseResolve).

  PerformPromiseRace

  Repeat
    ...
    Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], resultCapability.[[Reject]] »).

flags: [async]
features: [arrow-function]
includes: [promiseHelper.js]
---*/

let sequence = [1];
let lateRejector = {
  then(resolve, reject) {
    return new Promise((resolve) => {
      sequence.push(3);
      resolve();
      sequence.push(4);
    }).then(() => {
      sequence.push(5);
      resolve(9);
      sequence.push(6);
      reject();
      sequence.push(7);
    });
  }
};
sequence.push(2);

Promise.race([lateRejector])
  .then(resolution => {
    assert.sameValue(resolution, 9);
    assert.sameValue(sequence.length, 7);
    checkSequence(sequence);
  }).then($DONE, $DONE);
