// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.defer
description: Throws a TypeError if this does not have a [[DisposableState]] internal slot
info: |
  DisposableStack.prototype.defer ( )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  2. If O does not have an internalSlot internal slot, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof DisposableStack.prototype.defer, 'function');

var defer = DisposableStack.prototype.defer;

assert.throws(TypeError, function() {
  defer.call({ ['[[DisposableState]]']: {} });
}, 'Ordinary object without [[DisposableState]]');

assert.throws(TypeError, function() {
  defer.call(DisposableStack.prototype);
}, 'DisposableStack.prototype does not have a [[DisposableState]] internal slot');

assert.throws(TypeError, function() {
  defer.call(DisposableStack);
}, 'DisposableStack does not have a [[DisposableState]] internal slot');

var asyncStack = new AsyncDisposableStack(function() {});
assert.throws(TypeError, function() {
  defer.call(asyncStack);
}, 'AsyncDisposableStack instance');
