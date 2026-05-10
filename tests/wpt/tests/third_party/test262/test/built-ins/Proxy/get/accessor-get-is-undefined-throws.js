// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 9.5.8
description: >
    [[Get]] (P, Receiver)

    if trap result is not undefined, then proxy must report the same value for a
    non-configurable accessor property with an undefined get.
info: |
    13. If targetDesc is not undefined, then
        b. If IsAccessorDescriptor(targetDesc) and targetDesc.[[Configurable]]
        is false and targetDesc.[[Get]] is undefined, then
            i. If trapResult is not undefined, throw a TypeError exception.

features: [Proxy]
---*/

var target = {};
var p = new Proxy(target, {
  get: function() {
    return 2;
  }
});

Object.defineProperty(target, 'attr', {
  configurable: false,
  get: undefined
});

assert.throws(TypeError, function() {
  p.attr;
});

assert.throws(TypeError, function() {
  p['attr'];
});
