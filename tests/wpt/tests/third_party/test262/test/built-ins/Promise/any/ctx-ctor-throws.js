// Copyright (C) 2019 Sergey Rubanov. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Promise.any invoked on a constructor value that throws an error
esid: sec-promise.any
info: |
  2. Let promiseCapability be ? NewPromiseCapability(C).

  NewPromiseCapability

  ...
  7. Let promise be ? Construct(C, « executor »).

features: [Promise.any]
---*/

function CustomPromise() {
  throw new Test262Error();
}

assert.throws(Test262Error, function() {
  Promise.any.call(CustomPromise);
});
