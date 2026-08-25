// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-get-disposablestack.prototype.disposed
description: >
  Throws a TypeError if `this` is not an Object.
info: |
  get DisposableStack.prototype.disposed

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  ...

features: [explicit-resource-management,Symbol]
---*/

var descriptor = Object.getOwnPropertyDescriptor(DisposableStack.prototype, 'disposed');

assert.throws(TypeError, function() {
  descriptor.get.call(1);
});

assert.throws(TypeError, function() {
  descriptor.get.call(false);
});

assert.throws(TypeError, function() {
  descriptor.get.call(1);
});

assert.throws(TypeError, function() {
  descriptor.get.call('');
});

assert.throws(TypeError, function() {
  descriptor.get.call(undefined);
});

assert.throws(TypeError, function() {
  descriptor.get.call(null);
});

assert.throws(TypeError, function() {
  descriptor.get.call(Symbol());
});
