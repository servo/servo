// Copyright 2014 Cubane Canada, Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-promise-executor
author: Sam Mikes
description: >
  Promise constructor throws TypeError if executor is not callable.
info: |
  25.6.3.1 Promise ( executor )

  [...]
  2. If IsCallable(executor) is false, throw a TypeError exception.
---*/

assert.throws(TypeError, function() {
  new Promise('not callable');
});

assert.throws(TypeError, function() {
  new Promise(1);
});

assert.throws(TypeError, function() {
  new Promise(null);
});

assert.throws(TypeError, function() {
  new Promise({});
});
