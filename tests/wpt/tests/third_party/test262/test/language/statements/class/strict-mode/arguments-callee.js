// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class strict mode
---*/
var D = class extends function() {
  arguments.callee;
} {};
assert.throws(TypeError, function() {
  Object.getPrototypeOf(D).arguments;
});
assert.throws(TypeError, function() {
  new D;
});
