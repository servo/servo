// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Promise.try produces instances of the receiver for errors
esid: sec-promise.try
features: [promise-try, class]
flags: [async]
includes: [asyncHelpers.js]
---*/

class SubPromise extends Promise {
  constructor(executor) {
    super(executor);
  }
}

var error = {};
var instance = SubPromise.try(function () {
  throw error;
});
assert(instance instanceof SubPromise);

asyncTest(function() {
  return instance.then(function () {
    throw new Test262Error('Promise.try given a throwing function should throw');
  }, function (observedError) {
    assert.sameValue(observedError, error);
  });
});
