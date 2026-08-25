// Copyright (C) 2019 Leo Balter, 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise.any
description: Rejecting through deferred invocation of the provided resolving function
info: |
  ...
  5. Let result be PerformPromiseAny(iteratorRecord, C, promiseCapability).
  ...


flags: [async]
features: [AggregateError, Promise.any, arrow-function]
---*/

var rejection = {};
var thenable = {
  then(_, reject) {
    new Promise((resolve) => resolve())
      .then(() => reject(rejection));
  }
};

Promise.any([thenable])
  .then(() => {
    $DONE('The promise should be rejected.');
  }, (aggregate) => {
    assert(aggregate instanceof AggregateError);
    assert.sameValue(aggregate.errors.length, 1);
    assert.sameValue(aggregate.errors[0], rejection);
  }).then($DONE, $DONE);
