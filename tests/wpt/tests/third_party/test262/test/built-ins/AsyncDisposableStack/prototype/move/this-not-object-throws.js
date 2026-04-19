// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.move
description: Throws a TypeError if this is not an Object
info: |
  AsyncDisposableStack.prototype.move ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof AsyncDisposableStack.prototype.move, 'function');

var move = AsyncDisposableStack.prototype.move;

assert.throws(TypeError, function() {
  move.call(undefined);
}, 'undefined');

assert.throws(TypeError, function() {
  move.call(null);
}, 'null');

assert.throws(TypeError, function() {
  move.call(true);
}, 'true');

assert.throws(TypeError, function() {
  move.call(false);
}, 'false');

assert.throws(TypeError, function() {
  move.call(1);
}, 'number');

assert.throws(TypeError, function() {
  move.call('object');
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  move.call(s);
}, 'symbol');
