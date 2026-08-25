// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Invoked on an object without an [[OriginalFlags]] internal slot
es6id: 21.2.5.12
info: |
    21.2.5.12 get RegExp.prototype.sticky

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
    3. If R does not have an [[OriginalFlags]] internal slot, throw a TypeError
       exception.
---*/

var sticky = Object.getOwnPropertyDescriptor(RegExp.prototype, 'sticky').get;

assert.throws(TypeError, function() {
  sticky.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  sticky.call([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  sticky.call(arguments);
}, 'arguments object');

assert.throws(TypeError, function() {
  sticky.call(() => {});
}, 'function object');
