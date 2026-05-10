// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-asyncdisposablestack
description: >
  Return abrupt from getting the NewTarget prototype
info: |
  AsyncDisposableStack ( )

  ...
  2. Let asyncDisposableStack be ? OrdinaryCreateFromConstructor(NewTarget, "%AsyncDisposableStack.prototype%", « [[AsyncDisposableState]], [[DisposeCapability]] »).
  3. Set asyncDisposableStack.[[AsyncDisposableState]] to pending.
  4. Set asyncDisposableStack.[[DisposeCapability]] to NewDisposeCapability().
  5. Return asyncDisposableStack.

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
  Reflect.construct(AsyncDisposableStack, [], newTarget);
});

assert.sameValue(calls, 1);
