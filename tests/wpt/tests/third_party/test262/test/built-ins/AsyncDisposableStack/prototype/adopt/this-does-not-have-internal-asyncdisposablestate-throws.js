// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.adopt
description: Throws a TypeError if this does not have a [[AsyncDisposableState]] internal slot
info: |
  AsyncDisposableStack.prototype.adopt ( value, onDisposeAsync )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  2. If O does not have an internalSlot internal slot, throw a TypeError exception.
  ...

features: [explicit-resource-management]
---*/

assert.sameValue(typeof AsyncDisposableStack.prototype.adopt, 'function');

var adopt = AsyncDisposableStack.prototype.adopt;

assert.throws(TypeError, function() {
  adopt.call({ ['[[AsyncDisposableState]]']: {} });
}, 'Ordinary object without [[AsyncDisposableState]]');

assert.throws(TypeError, function() {
  adopt.call(AsyncDisposableStack.prototype);
}, 'AsyncDisposableStack.prototype does not have a [[AsyncDisposableState]] internal slot');

assert.throws(TypeError, function() {
  adopt.call(AsyncDisposableStack);
}, 'AsyncDisposableStack does not have a [[AsyncDisposableState]] internal slot');

var stack = new DisposableStack();
assert.throws(TypeError, function() {
  adopt.call(stack);
}, 'DisposableStack instance');
