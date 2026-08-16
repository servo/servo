// Copyright (C) 2026 Danial Asaria (Bloomberg LP). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-promise.allsettledkeyed
description: >
  Promise.allSettledKeyed invoked on a constructor value that throws an error
info: |
  Promise.allSettledKeyed ( promises )

  1. Let C be the this value.
  2. Let promiseCapability be ? NewPromiseCapability(C).

  NewPromiseCapability ( C )

  ...
  6. Let promise be Construct(C, « executor »).
  7. ReturnIfAbrupt(promise).
features: [await-dictionary]
---*/

var CustomPromise = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  Promise.allSettledKeyed.call(CustomPromise, {});
});
