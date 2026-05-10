// Copyright (C) 2019 Aleksey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise-executor
description: >
  Throws a TypeError if Promise is called without a NewTarget.
info: |
  25.6.3.1 Promise ( executor )

  1. If NewTarget is undefined, throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  Promise(function() {});
});

assert.throws(TypeError, function() {
  Promise.call(null, function() {});
});

var p = new Promise(function() {});
assert.throws(TypeError, function() {
  Promise.call(p, function() {});
});
