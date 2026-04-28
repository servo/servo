// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack.prototype.use
description: Throws if the argument has a non-null, non-undefined, non-callable Symbol.dispose property.
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

  CreateDisposableResource ( V, hint [ , method ] )

  1. If method is not present, then
    a. If V is either null or undefined, then
      i. Set V to undefined
      ii. Set method to undefined
    b. Else,
      i. If Type(V) is not Object, throw a TypeError exception.
      ii. Set method to ? GetDisposeMethod(V, hint).
      iii. If method is undefined, throw a TypeError exception.
  2. Else,
      a. ...
  3. Return the DisposableResource Record { [[ResourceValue]]: V, [[Hint]]: hint, [[DisposeMethod]]: method }.

  GetDisposeMethod ( V, hint )

  1. If hint is async-dispose, then
    a. Let method be ? GetMethod(V, @@asyncDispose).
    b. If method is undefined, then
      i. Set method to ? GetMethod(V, @@dispose).
  2. Else,
    a. Let method be ? GetMethod(V, @@dispose).
  ...

  GetMethod ( V, P )

    1. Let func be ? GetV(V, P).
    2. If func is either undefined or null, return undefined.
    3. If IsCallable(func) is false, throw a TypeError exception.
    ...

features: [explicit-resource-management]
---*/

var stack = new DisposableStack();
assert.throws(TypeError, function() {
  stack.use({ [Symbol.dispose]: true });
}, 'true');

assert.throws(TypeError, function() {
  stack.use({ [Symbol.dispose]: false });
}, 'false');

assert.throws(TypeError, function() {
  stack.use({ [Symbol.dispose]: 1 });
}, 'number');

assert.throws(TypeError, function() {
  stack.use({ [Symbol.dispose]: 'object' });
}, 'string');

var s = Symbol();
assert.throws(TypeError, function() {
  stack.use({ [Symbol.dispose]: s });
}, 'symbol');
