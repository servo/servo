// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Promise.allSettled invoked on a constructor value that throws an error
esid: sec-promise.allsettled
info: |
  3. Let promiseCapability be ? NewPromiseCapability(C).

  NewPromiseCapability

  ...
  7. Let promise be ? Construct(C, « executor »).
features: [Promise.allSettled]
---*/

var CustomPromise = function() {
  throw new Test262Error();
};

assert.throws(Test262Error, function() {
  Promise.allSettled.call(CustomPromise);
});
