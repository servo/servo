// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Resolved promises ignore rejections through immediate invocation of the
    provided resolving function
esid: sec-promise.allsettled
info: |
  Let result be PerformPromiseAllSettled(iteratorRecord, C, promiseCapability).

  Runtime Semantics: PerformPromiseAllSettled

  Repeat
    ...
    r. Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

flags: [async]
features: [Promise.allSettled, arrow-function]
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

Promise.allSettled([resolver, lateRejector])
  .then(resolution => {
    assert.sameValue(resolution[0].value, 42);
    assert.sameValue(resolution[0].status, 'fulfilled');
    assert.sameValue(resolution[1].value, 33);
    assert.sameValue(resolution[1].status, 'fulfilled');
  }).then($DONE, $DONE);
