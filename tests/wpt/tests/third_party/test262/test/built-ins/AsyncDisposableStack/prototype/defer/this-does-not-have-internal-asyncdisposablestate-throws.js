// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.defer
description: Throws a TypeError if this does not have a [[AsyncDisposableState]] internal slot
info: |
  AsyncDisposableStack.prototype.defer ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  2. If O does not have an internalSlot internal slot, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof AsyncDisposableStack.prototype.defer, 'function');

var defer = AsyncDisposableStack.prototype.defer;

assert.throws(TypeError, function() {
  defer.call({ ['[[AsyncDisposableState]]']: {} });
}, 'Ordinary object without [[AsyncDisposableState]]');

assert.throws(TypeError, function() {
  defer.call(AsyncDisposableStack.prototype);
}, 'AsyncDisposableStack.prototype does not have a [[AsyncDisposableState]] internal slot');

assert.throws(TypeError, function() {
  defer.call(AsyncDisposableStack);
}, 'AsyncDisposableStack does not have a [[AsyncDisposableState]] internal slot');

var stack = new DisposableStack();
assert.throws(TypeError, function() {
  defer.call(stack);
}, 'DisposableStack instance');
