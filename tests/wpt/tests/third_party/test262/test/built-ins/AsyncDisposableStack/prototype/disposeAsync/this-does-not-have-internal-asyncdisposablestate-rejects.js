// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.disposeAsync
description: Throws a TypeError if this does not have a [[AsyncDisposableState]] internal slot
info: |
  AsyncDisposableStack.prototype.disposeAsync ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  3. ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  2. If O does not have an internalSlot internal slot, throw a TypeError exception.
  ...

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

asyncTest(async function() {
  assert.sameValue(typeof AsyncDisposableStack.prototype.disposeAsync, 'function');
  
  var disposeAsync = AsyncDisposableStack.prototype.disposeAsync;
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call({ ['[[AsyncDisposableState]]']: {} });
  }, 'Ordinary object without [[AsyncDisposableState]]');
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(AsyncDisposableStack.prototype);
  }, 'AsyncDisposableStack.prototype does not have a [[AsyncDisposableState]] internal slot');
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(AsyncDisposableStack);
  }, 'AsyncDisposableStack does not have a [[AsyncDisposableState]] internal slot');
  
  var stack = new DisposableStack();
  await assert.throwsAsync(TypeError, function () {
    return disposeAsync.call(stack);
  }, 'DisposableStack instance');
});
