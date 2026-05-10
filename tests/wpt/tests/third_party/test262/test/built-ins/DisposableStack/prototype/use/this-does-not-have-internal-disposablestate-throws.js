// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.use
description: Throws a TypeError if this does not have a [[DisposableState]] internal slot
info: |
  DisposableStack.prototype.use ( value )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  4. Perform ? AddDisposableResource(disposableStack.[[DisposeCapability]], value, sync-dispose).
  5. Return value.

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  2. If O does not have an internalSlot internal slot, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof DisposableStack.prototype.use, 'function');

var use = DisposableStack.prototype.use;

assert.throws(TypeError, function() {
  use.call({ ['[[DisposableState]]']: {} });
}, 'Ordinary object without [[DisposableState]]');

assert.throws(TypeError, function() {
  use.call(DisposableStack.prototype);
}, 'DisposableStack.prototype does not have a [[DisposableState]] internal slot');

assert.throws(TypeError, function() {
  use.call(DisposableStack);
}, 'DisposableStack does not have a [[DisposableState]] internal slot');

var asyncStack = new AsyncDisposableStack(function() {});
assert.throws(TypeError, function() {
  use.call(asyncStack);
}, 'AsyncDisposableStack instance');
