// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack.prototype.disposeAsync
description: Throws a TypeError if this is not an Object
info: |
  AsyncDisposableStack.prototype.disposeAsync ( )

  1. Let asyncDisposableStack be the this value.
  2. Perform ? RequireInternalSlot(asyncDisposableStack, [[AsyncDisposableState]]).
  ...

  RequireInternalSlot ( O, internalSlot )

  1. If O is not an Object, throw a TypeError exception.
  ...

flags: [async]
includes: [asyncHelpers.js]
features: [explicit-resource-management]
---*/

assert.sameValue(typeof AsyncDisposableStack.prototype.disposeAsync, 'function');

var disposeAsync = AsyncDisposableStack.prototype.disposeAsync;

asyncTest(async function () {
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(undefined);
  }, 'undefined');
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(null);
  }, 'null');
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(true);
  }, 'true');
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(false);
  }, 'false');
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(1);
  }, 'number');
  
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call('object');
  }, 'string');
  
  var s = Symbol();
  await assert.throwsAsync(TypeError, function() {
    return disposeAsync.call(s);
  }, 'symbol');
});
