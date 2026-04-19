// Copyright (C) 2020 Rick Waldron, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Resolved promises ignore rejections through immediate invocation of the
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
---*/

let resolver = {
  then(resolve) {
    resolve(42);
  }
};
let lateRejector = {
  then(resolve, reject) {
    resolve(33);
    reject();
  }
};

Promise.race([resolver, lateRejector])
  .then(resolution => {
    assert.sameValue(resolution, 42);
  }).then($DONE, $DONE);
