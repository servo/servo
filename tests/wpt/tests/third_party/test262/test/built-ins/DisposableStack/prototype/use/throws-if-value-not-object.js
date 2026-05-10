// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.use
description: Throws if the argument is not an object and is neither null nor undefined.
info: |
  DisposableStack.prototype.use ( value )

  1. Let disposableStack be the this value.
  2. Perform ? RequireInternalSlot(disposableStack, [[DisposableState]]).
  3. If disposableStack.[[DisposableState]] is disposed, throw a ReferenceError exception.
  4. Perform ? AddDisposableResource(disposableStack.[[DisposeCapability]], value, sync-dispose).
  ...

  AddDisposableResource ( disposeCapability, V, hint [, method ] )

  1. If method is not present then,
    a. If V is either null or undefined and hint is sync-dispose, then
      i. Return unused
    b. Let resource be ? CreateDisposableResource(V, hint).
    ...
  ...

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
assert.throws(TypeError, function() {
  stack.use(true);
}, 'true');

assert.throws(TypeError, function() {
  stack.use(false);
}, 'false');

assert.throws(TypeError, function() {
  stack.use(1);
}, 'number');

assert.throws(TypeError, function() {
  stack.use('object');
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  stack.use(s);
}, 'symbol');
