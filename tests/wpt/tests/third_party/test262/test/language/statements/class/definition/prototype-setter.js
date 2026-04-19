// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class prototype setter
---*/
var Base = function() {}.bind();
Object.defineProperty(Base, 'prototype', {
  set: function() {
    throw new Test262Error("Cannot create a setter property named `prototype`");
  }
});
assert.throws(TypeError, function() {
  class C extends Base {}
});
