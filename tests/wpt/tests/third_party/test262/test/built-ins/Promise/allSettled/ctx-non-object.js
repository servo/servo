// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
  Promise.allSettled invoked on a non-object value
esid: sec-promise.allsettled
info: |
  1. Let C be the this value.
  2. If Type(C) is not Object, throw a TypeError exception.
features: [Promise.allSettled, Symbol]
---*/

assert.throws(TypeError, function() {
  Promise.allSettled.call(undefined, []);
});

assert.throws(TypeError, function() {
  Promise.allSettled.call(null, []);
});

assert.throws(TypeError, function() {
  Promise.allSettled.call(86, []);
});

assert.throws(TypeError, function() {
  Promise.allSettled.call('string', []);
});

assert.throws(TypeError, function() {
  Promise.allSettled.call(true, []);
});

assert.throws(TypeError, function() {
  Promise.allSettled.call(Symbol(), []);
});
