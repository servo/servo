// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    `unicode` accessor invoked on a non-object value
es6id: 21.2.5.15
info: |
    21.2.5.15 get RegExp.prototype.unicode

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
features: [Symbol]
---*/

var unicode = Object.getOwnPropertyDescriptor(RegExp.prototype, 'unicode').get;

assert.throws(TypeError, function() {
  unicode.call(undefined);
});

assert.throws(TypeError, function() {
  unicode.call(null);
});

assert.throws(TypeError, function() {
  unicode.call(true);
});

assert.throws(TypeError, function() {
  unicode.call('string');
});

assert.throws(TypeError, function() {
  unicode.call(Symbol('s'));
});

assert.throws(TypeError, function() {
  unicode.call(4);
});

assert.throws(TypeError, function() {
  unicode.call(4n);
}, 'bigint');
