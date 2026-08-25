// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: >
    Behavior when "this" value is an Object without a [[RegExpMatcher]]
    internal slot
info: |
    1. Let O be the this value.
    2. If Type(O) is not Object or Type(O) is Object and O does not have a
       [[RegExpMatcher]] internal slot, then
       a. Throw a TypeError exception.
---*/

var compile = RegExp.prototype.compile;

assert.sameValue(typeof compile, 'function');

assert.throws(TypeError, function() {
  compile.call({});
}, 'ordinary object');

assert.throws(TypeError, function() {
  compile.call([]);
}, 'array exotic object');

assert.throws(TypeError, function() {
  compile.call(arguments);
}, 'arguments exotic object');
