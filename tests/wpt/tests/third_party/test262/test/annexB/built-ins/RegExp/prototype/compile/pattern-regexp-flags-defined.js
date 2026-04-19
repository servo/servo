// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-regexp.prototype.compile
es6id: B.2.5.1
description: >
    Behavior when provided pattern is a RegExp instance and flags are specified
info: |
    [...]
    3. If Type(pattern) is Object and pattern has a [[RegExpMatcher]] internal
       slot, then
       a. If flags is not undefined, throw a TypeError exception.
---*/

var re = /./;
re.lastIndex = 23;

assert.sameValue(typeof RegExp.prototype.compile, 'function');

assert.throws(TypeError, function() {
  re.compile(re, null);
}, 'null');

assert.throws(TypeError, function() {
  re.compile(re, 0);
}, 'numeric primitive');

assert.throws(TypeError, function() {
  re.compile(re, '');
}, 'string primitive');

assert.throws(TypeError, function() {
  re.compile(re, false);
}, 'boolean primitive');

assert.throws(TypeError, function() {
  re.compile(re, {});
}, 'ordinary object');

assert.throws(TypeError, function() {
  re.compile(re, []);
}, 'array exotic object');

assert.sameValue(re.lastIndex, 23);
