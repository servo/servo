// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.dotall
description: Invoked on an object without an [[OriginalFlags]] internal slot
info: |
    get RegExp.prototype.dotAll

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
    3. If R does not have an [[OriginalFlags]] internal slot, then
      a. If SameValue(R, %RegExpPrototype%) is true, return undefined.
      b. Otherwise, throw a TypeError exception.
features: [regexp-dotall]
---*/

var dotAll = Object.getOwnPropertyDescriptor(RegExp.prototype, 'dotAll').get;

assert.throws(TypeError, function() {
  dotAll.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  dotAll.call([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  dotAll.call(arguments);
}, 'arguments object');

assert.throws(TypeError, function() {
  dotAll.call(() => {});
}, 'function object');
