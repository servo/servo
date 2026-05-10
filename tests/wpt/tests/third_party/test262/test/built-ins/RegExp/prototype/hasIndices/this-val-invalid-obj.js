// Copyright (C) 2021 Ron Buckton and the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-get-regexp.prototype.hasindices
description: Invoked on an object without an [[OriginalFlags]] internal slot
info: |
    get RegExp.prototype.hasIndices

    1. Let R be the this value.
    2. If Type(R) is not Object, throw a TypeError exception.
    3. If R does not have an [[OriginalFlags]] internal slot, then
      a. If SameValue(R, %RegExpPrototype%) is true, return undefined.
      b. Otherwise, throw a TypeError exception.
features: [regexp-match-indices]
---*/

var hasIndices = Object.getOwnPropertyDescriptor(RegExp.prototype, 'hasIndices').get;

assert.throws(TypeError, function() {
  hasIndices.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  hasIndices.call([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  hasIndices.call(arguments);
}, 'arguments object');

assert.throws(TypeError, function() {
  hasIndices.call(() => {});
}, 'function object');
