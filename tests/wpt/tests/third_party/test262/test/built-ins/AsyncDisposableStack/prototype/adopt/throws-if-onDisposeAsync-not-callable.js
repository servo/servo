// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.adopt
description: Throws if onDisposeAsync argument not callable
info: |
  AsyncDisposableStack.prototype.adopt ( value, onDisposeAsync )

  ...
  4. If IsCallable(onDisposeAsync) is false, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

var stack = new AsyncDisposableStack();
assert.throws(TypeError, function() {
  stack.adopt(null, null);
}, 'null');

assert.throws(TypeError, function() {
  stack.adopt(null, undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  stack.adopt(null, true);
}, 'true');

assert.throws(TypeError, function() {
  stack.adopt(null, false);
}, 'false');

assert.throws(TypeError, function() {
  stack.adopt(null, 1);
}, 'number');

assert.throws(TypeError, function() {
  stack.adopt(null, 'object');
}, 'string');

assert.throws(TypeError, function() {
  stack.adopt(null, {});
}, 'object');

var s = Symbol();
assert.throws(TypeError, function() {
  stack.adopt(null, s);
}, 'symbol');
