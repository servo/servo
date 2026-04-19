// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Resolving with a thenable object value
esid: sec-promise.allsettled
info: |
  Let promiseCapability be NewPromiseCapability(C).
flags: [async]
features: [Promise.allSettled]
---*/

var value = {};
var promise;

try {
  Array.prototype.then = function(resolve) {
    resolve(value);
  };

  promise = Promise.allSettled([]);
} finally {
  delete Array.prototype.then;
}

promise.then(function(val) {
  assert.sameValue(val, value);
}, function() {
  $DONE('The promise should not be rejected.');
}).then($DONE, $DONE);
