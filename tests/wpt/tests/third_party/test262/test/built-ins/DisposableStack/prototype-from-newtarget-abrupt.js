// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-disposablestack
description: >
  Return abrupt from getting the NewTarget prototype
info: |
  DisposableStack ( )

  ...
  2. Let disposableStack be ? OrdinaryCreateFromConstructor(NewTarget, "%DisposableStack.prototype%", « [[DisposableState]], [[DisposeCapability]] »).
  3. Set disposableStack.[[DisposableState]] to pending.
  4. Set disposableStack.[[DisposeCapability]] to NewDisposeCapability().
  5. Return disposableStack.

  OrdinaryCreateFromConstructor ( constructor, intrinsicDefaultProto [ , internalSlotsList ] )

  ...
  2. Let proto be ? GetPrototypeFromConstructor(constructor, intrinsicDefaultProto).
  3. Return ObjectCreate(proto, internalSlotsList).

  GetPrototypeFromConstructor ( constructor, intrinsicDefaultProto )

  3. Let proto be ? Get(constructor, 'prototype').
features: [explicit-resource-management, Reflect.construct]
---*/

var calls = 0;
var newTarget = function() {}.bind(null);
Object.defineProperty(newTarget, 'prototype', {
  get: function() {
    calls += 1;
    throw new Test262Error();
  }
});

assert.throws(Test262Error, function() {
  Reflect.construct(DisposableStack, [], newTarget);
});

assert.sameValue(calls, 1);
