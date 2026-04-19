// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.defer
description: Adds a callback to the stack
info: |
  DisposableStack.prototype.defer ( onDispose )

  ...
  4. If IsCallable(onDispose) is false, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
assert.throws(TypeError, function() {
  stack.defer(null);
}, 'null');

assert.throws(TypeError, function() {
  stack.defer(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  stack.defer(true);
}, 'true');

assert.throws(TypeError, function() {
  stack.defer(false);
}, 'false');

assert.throws(TypeError, function() {
  stack.defer(1);
}, 'number');

assert.throws(TypeError, function() {
  stack.defer('object');
}, 'string');

assert.throws(TypeError, function() {
  stack.defer({});
}, 'object');

var s = Symbol();
assert.throws(TypeError, function() {
  stack.defer(s);
}, 'symbol');
