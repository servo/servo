// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Promise.allSettled invoked on a non-constructor value
esid: sec-promise.allsettled
info: |
  ...
  3. Let promiseCapability be ? NewPromiseCapability(C).

  NewPromiseCapability ( C )

  1. If IsConstructor(C) is false, throw a TypeError exception.
features: [Promise.allSettled]
---*/

assert.throws(TypeError, function() {
  Promise.allSettled.call(eval);
});
