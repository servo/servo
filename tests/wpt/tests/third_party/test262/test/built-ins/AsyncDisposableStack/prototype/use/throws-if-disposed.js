// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.use
description: Throws a ReferenceError if this is disposed.
info: |
  AsyncDisposableStack.prototype.use ( value )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a ReferenceError exception.
  ...

features: [explicit-resource-management]
---*/

var stack = new AsyncDisposableStack();
stack.disposeAsync();

assert.throws(ReferenceError, function() {
  stack.use(undefined);
}, 'undefined');

assert.throws(ReferenceError, function() {
  stack.use(null);
}, 'null');

assert.throws(ReferenceError, function() {
  stack.use(true);
}, 'true');

assert.throws(ReferenceError, function() {
  stack.use(false);
}, 'false');

assert.throws(ReferenceError, function() {
  stack.use(1);
}, 'number');

assert.throws(ReferenceError, function() {
  stack.use('object');
}, 'string');

var s = Symbol();
assert.throws(ReferenceError, function() {
  stack.use(s);
}, 'symbol');

assert.throws(ReferenceError, function() {
  stack.use({});
}, 'non disposable object');

assert.throws(ReferenceError, function() {
  stack.use({ async [Symbol.asyncDispose]() {} });
}, 'async disposable object');

assert.throws(ReferenceError, function() {
  stack.use({ [Symbol.dispose]() {} });
}, 'disposable object');
