// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.move
description: Throws a TypeError if this does not have a [[AsyncDisposableState]] internal slot
info: |
  AsyncDisposableStack.prototype.move ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  2. If O does not have an internalSlot internal slot, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof AsyncDisposableStack.prototype.move, 'function');

var move = AsyncDisposableStack.prototype.move;

assert.throws(TypeError, function() {
  move.call({ ['[[AsyncDisposableState]]']: {} });
}, 'Ordinary object without [[AsyncDisposableState]]');

assert.throws(TypeError, function() {
  move.call(AsyncDisposableStack.prototype);
}, 'AsyncDisposableStack.prototype does not have a [[AsyncDisposableState]] internal slot');

assert.throws(TypeError, function() {
  move.call(AsyncDisposableStack);
}, 'AsyncDisposableStack does not have a [[AsyncDisposableState]] internal slot');

var stack = new DisposableStack();
assert.throws(TypeError, function() {
  move.call(stack);
}, 'DisposableStack instance');
