// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Resolved promises ignore rejections through immediate invocation of the
    provided resolving function
esid: sec-promise.any
info: |
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAny

  8. Repeat
    ...
    r. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

flags: [async]
features: [Promise.any, arrow-function]
---*/

var resolver = {
  then(resolve) {
    resolve(42);
  }
};
var lateRejector = {
  then(resolve, reject) {
    resolve(33);
    reject();
  }
};

Promise.any([resolver, lateRejector])
  .then(resolution => {
    assert.sameValue(resolution, 42);
  }).then($DONE, $DONE);
