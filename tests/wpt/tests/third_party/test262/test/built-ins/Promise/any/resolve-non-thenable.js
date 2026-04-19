// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a non-thenable object value
esid: sec-promise.any
info: |
  PerformPromiseAny

  Set remainingElementsCount.[[Value]] to remainingElementsCount.[[Value]] + 1.
  Perform ? Invoke(nextPromise, "then", « resultCapability.[[Resolve]], rejectElement »).

flags: [async]
features: [Promise.any]
---*/

const a = {};
const b = {};
const c = {};

Promise.any([a, b, c])
  .then((value) => {
    assert.sameValue(value, a);
  }, () => {
    $DONE('The promise should not be rejected.');
  }).then($DONE, $DONE);
