// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `sticky` accessor invoked on a non-object value
es6id: 21.2.5.12
info: |
    21.2.5.12 get RegExp.prototype.sticky

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
features: [Symbol]
---*/

var sticky = Object.getOwnPropertyDescriptor(RegExp.prototype, 'sticky').get;

assert.throws(TypeError, function() {
  sticky.call(undefined);
});

assert.throws(TypeError, function() {
  sticky.call(null);
});

assert.throws(TypeError, function() {
  sticky.call(true);
});

assert.throws(TypeError, function() {
  sticky.call('string');
});

assert.throws(TypeError, function() {
  sticky.call(Symbol('s'));
});

assert.throws(TypeError, function() {
  sticky.call(4);
});

assert.throws(TypeError, function() {
  sticky.call(4n);
}, 'bigint');
